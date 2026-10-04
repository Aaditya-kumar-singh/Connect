use crate::{
    app_state::AppState,
    auth::models::AuthenticatedSession,
    domain::errors::AppError,
    messaging::{
        models::{Message, MessageHistory},
        repository, validation,
    },
};
use chrono::{Duration, Utc};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

async fn require_member(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    conversation_id: Uuid,
) -> Result<(), AppError> {
    let member = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1 FROM conversation_members
            WHERE conversation_id=$1 AND user_id=$2 AND left_at IS NULL
        )",
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await?;
    if !member {
        return Err(AppError::Forbidden(
            "You are not a member of this conversation".into(),
        ));
    }
    Ok(())
}

async fn require_member_pool(
    state: &AppState,
    user_id: Uuid,
    conversation_id: Uuid,
) -> Result<(), AppError> {
    let member = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1 FROM conversation_members
            WHERE conversation_id=$1 AND user_id=$2 AND left_at IS NULL
        )",
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_one(state.db())
    .await?;
    if !member {
        return Err(AppError::Forbidden(
            "You are not a member of this conversation".into(),
        ));
    }
    Ok(())
}

pub async fn send(
    state: &AppState,
    auth: &AuthenticatedSession,
    input: &crate::messaging::models::SendMessagePayload,
) -> Result<(Message, bool), AppError> {
    validation::validate_text(&input.content)?;
    validation::validate_content_type(&input.content_type)?;

    let mut tx = state.db().begin().await?;
    require_member(&mut tx, auth.user_id, input.conversation_id).await?;

    if let Some(reply_id) = input.reply_to_message_id {
        let valid = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1 FROM messages
                WHERE id=$1 AND conversation_id=$2 AND deleted_at IS NULL
            )",
        )
        .bind(reply_id)
        .bind(input.conversation_id)
        .fetch_one(&mut *tx)
        .await?;
        if !valid {
            return Err(AppError::Validation(
                "Reply target is not in this conversation".into(),
            ));
        }
    }

    let (message, created) = repository::insert_idempotent(
        &mut tx,
        repository::NewMessage {
            message_id: Uuid::new_v4(),
            client_message_id: input.client_message_id,
            conversation_id: input.conversation_id,
            sender_id: auth.user_id,
            content: &input.content,
            content_type: &input.content_type,
            reply_to_message_id: input.reply_to_message_id,
            forwarded_from_message_id: None,
        },
    )
    .await?;

    if !created && message.sender_id != Some(auth.user_id) {
        return Err(AppError::Conflict(
            "client_message_id is already used".into(),
        ));
    }

    if created {
        sqlx::query("UPDATE conversations SET updated_at=$2 WHERE id=$1")
            .bind(input.conversation_id)
            .bind(message.created_at)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok((message, created))
}

pub async fn history(
    state: &AppState,
    auth: &AuthenticatedSession,
    conversation_id: Uuid,
    limit: Option<u8>,
    cursor: Option<&str>,
) -> Result<MessageHistory, AppError> {
    require_member_pool(state, auth.user_id, conversation_id).await?;
    let limit = validation::validate_limit(limit)?;
    let decoded = validation::decode_cursor(cursor)?
        .map(|(timestamp, id)| {
            chrono::DateTime::parse_from_rfc3339(&timestamp)
                .map(|value| (value.with_timezone(&Utc), id))
                .map_err(|_| AppError::Validation("Invalid cursor".into()))
        })
        .transpose()?;

    let rows = repository::history(state.db(), conversation_id, limit + 1, decoded).await?;
    let has_more = rows.len() > limit as usize;
    let mut items = rows;
    if has_more {
        items.pop();
    }
    let next_cursor = if has_more {
        items
            .last()
            .map(|m| validation::encode_cursor(&m.created_at.to_rfc3339(), m.id))
    } else {
        None
    };

    Ok(MessageHistory { items, next_cursor })
}

pub async fn edit(
    state: &AppState,
    auth: &AuthenticatedSession,
    message_id: Uuid,
    content: &str,
) -> Result<Message, AppError> {
    validation::validate_text(content)?;
    let mut tx = state.db().begin().await?;
    let message = repository::find(state.db(), message_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Message not found".into()))?;
    if message.sender_id != Some(auth.user_id) {
        return Err(AppError::Forbidden(
            "Only the message sender can edit it".into(),
        ));
    }
    require_member(&mut tx, auth.user_id, message.conversation_id).await?;
    if message.deleted_at.is_some() {
        return Err(AppError::Conflict(
            "Deleted messages cannot be edited".into(),
        ));
    }
    if message.created_at + Duration::minutes(15) < Utc::now() {
        return Err(AppError::Validation(
            "Message edit window has expired".into(),
        ));
    }
    let old = message.content.unwrap_or_default();
    let now = Utc::now();
    let updated = repository::edit(&mut tx, message_id, &old, content, now).await?;
    sqlx::query("UPDATE conversations SET updated_at=$2 WHERE id=$1")
        .bind(message.conversation_id)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(updated)
}

pub async fn delete(
    state: &AppState,
    auth: &AuthenticatedSession,
    message_id: Uuid,
) -> Result<Message, AppError> {
    let mut tx = state.db().begin().await?;
    let message = repository::find(state.db(), message_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Message not found".into()))?;
    if message.sender_id != Some(auth.user_id) {
        return Err(AppError::Forbidden(
            "Only the message sender can delete it".into(),
        ));
    }
    require_member(&mut tx, auth.user_id, message.conversation_id).await?;
    if message.deleted_at.is_some() {
        return Err(AppError::Conflict("Message is already deleted".into()));
    }
    let now = Utc::now();
    let updated = repository::delete(&mut tx, message_id, now).await?;
    sqlx::query("UPDATE conversations SET updated_at=$2 WHERE id=$1")
        .bind(message.conversation_id)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(updated)
}

pub async fn react(
    state: &AppState,
    auth: &AuthenticatedSession,
    message_id: Uuid,
    emoji: &str,
    action: &str,
) -> Result<(Uuid, Uuid, Uuid, String, String, bool), AppError> {
    validation::validate_emoji(emoji)?;
    validation::validate_reaction_action(action)?;
    let mut tx = state.db().begin().await?;
    let message = repository::find(state.db(), message_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Message not found".into()))?;
    require_member(&mut tx, auth.user_id, message.conversation_id).await?;
    if message.deleted_at.is_some() {
        return Err(AppError::Conflict(
            "Deleted messages cannot be reacted to".into(),
        ));
    }

    let changed = if action == "add" {
        repository::add_reaction(&mut tx, message_id, auth.user_id, emoji).await?
    } else {
        repository::remove_reaction(&mut tx, message_id, auth.user_id, emoji).await?
    };
    tx.commit().await?;
    Ok((
        message_id,
        message.conversation_id,
        auth.user_id,
        emoji.to_owned(),
        action.to_owned(),
        changed,
    ))
}

pub async fn delivered(
    state: &AppState,
    auth: &AuthenticatedSession,
    message_ids: &[Uuid],
) -> Result<Vec<(Uuid, Uuid, chrono::DateTime<Utc>)>, AppError> {
    validation::validate_message_ids(message_ids)?;
    let mut unique_ids = Vec::with_capacity(message_ids.len());
    let mut seen = std::collections::HashSet::with_capacity(message_ids.len());
    for message_id in message_ids {
        if seen.insert(*message_id) {
            unique_ids.push(*message_id);
        }
    }
    let mut tx = state.db().begin().await?;
    let statuses = repository::record_delivered(&mut tx, auth.user_id, &unique_ids).await?;
    tx.commit().await?;
    Ok(statuses)
}

pub async fn read(
    state: &AppState,
    auth: &AuthenticatedSession,
    conversation_id: Uuid,
    last_read_message_id: Uuid,
) -> Result<Vec<(Uuid, Uuid, chrono::DateTime<Utc>)>, AppError> {
    let mut tx = state.db().begin().await?;
    require_member(&mut tx, auth.user_id, conversation_id).await?;
    let statuses =
        repository::record_read(&mut tx, auth.user_id, conversation_id, last_read_message_id)
            .await?;
    tx.commit().await?;
    Ok(statuses)
}

pub async fn forward(
    state: &AppState,
    auth: &AuthenticatedSession,
    input: &crate::messaging::models::ForwardMessageRequest,
) -> Result<Message, AppError> {
    let source = repository::find(state.db(), input.message_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Message not found".into()))?;
    if source.deleted_at.is_some() || source.content_type != "text" {
        return Err(AppError::Conflict(
            "Only active text messages can be forwarded".into(),
        ));
    }
    require_member_pool(state, auth.user_id, source.conversation_id).await?;
    require_member_pool(state, auth.user_id, input.conversation_id).await?;

    let content = source
        .content
        .clone()
        .ok_or_else(|| AppError::Conflict("Message has no content".into()))?;
    let mut tx = state.db().begin().await?;
    let (message, created) = repository::insert_idempotent(
        &mut tx,
        repository::NewMessage {
            message_id: Uuid::new_v4(),
            client_message_id: input.client_message_id,
            conversation_id: input.conversation_id,
            sender_id: auth.user_id,
            content: &content,
            content_type: "text",
            reply_to_message_id: None,
            forwarded_from_message_id: Some(source.id),
        },
    )
    .await?;

    if !created && message.sender_id != Some(auth.user_id) {
        return Err(AppError::Conflict(
            "client_message_id is already used".into(),
        ));
    }
    sqlx::query("UPDATE conversations SET updated_at=$2 WHERE id=$1")
        .bind(input.conversation_id)
        .bind(message.created_at)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(message)
}
