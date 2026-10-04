use crate::{
    app_state::AppState, auth::models::AuthenticatedSession, domain::errors::AppError,
    websocket::pubsub,
};
use axum::extract::ws::Message;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const TYPING_TTL_SECONDS: usize = 5;

#[derive(Debug, Clone, Deserialize)]
pub struct TypingPayload {
    pub conversation_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypingUpdatePayload {
    pub conversation_id: Uuid,
    pub user_id: Uuid,
    pub display_name: String,
}

pub async fn start(
    state: &AppState,
    auth: &AuthenticatedSession,
    conversation_id: Uuid,
) -> Result<String, AppError> {
    ensure_member(state, auth.user_id, conversation_id).await?;
    let rate_key = format!("rate:typing:{}:{}", auth.user_id, conversation_id);
    crate::auth::rate_limit::check_key(state, &rate_key, 1, 2).await?;

    let key = typing_key(conversation_id, auth.user_id);
    let mut conn = state.redis().get_multiplexed_async_connection().await?;
    let _: () = redis::cmd("SET")
        .arg(&key)
        .arg("typing")
        .arg("EX")
        .arg(TYPING_TTL_SECONDS)
        .query_async(&mut conn)
        .await?;

    let display_name = sqlx::query_scalar::<_, String>(
        "SELECT display_name FROM users WHERE id=$1 AND status='ACTIVE'",
    )
    .bind(auth.user_id)
    .fetch_optional(state.db())
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    let frame = frame(
        "typing.start",
        TypingUpdatePayload {
            conversation_id,
            user_id: auth.user_id,
            display_name,
        },
    );
    broadcast(state, conversation_id, auth.user_id, &frame).await?;
    Ok(frame)
}

pub async fn stop(
    state: &AppState,
    auth: &AuthenticatedSession,
    conversation_id: Uuid,
) -> Result<Option<String>, AppError> {
    ensure_member(state, auth.user_id, conversation_id).await?;
    let key = typing_key(conversation_id, auth.user_id);
    let mut conn = state.redis().get_multiplexed_async_connection().await?;
    let deleted: i64 = redis::cmd("DEL").arg(&key).query_async(&mut conn).await?;
    if deleted == 0 {
        return Ok(None);
    }

    let display_name = sqlx::query_scalar::<_, String>(
        "SELECT display_name FROM users WHERE id=$1 AND status='ACTIVE'",
    )
    .bind(auth.user_id)
    .fetch_optional(state.db())
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    let frame = frame(
        "typing.stop",
        TypingUpdatePayload {
            conversation_id,
            user_id: auth.user_id,
            display_name,
        },
    );
    broadcast(state, conversation_id, auth.user_id, &frame).await?;
    Ok(Some(frame))
}

pub fn frame(event_type: &str, payload: TypingUpdatePayload) -> String {
    serde_json::json!({
        "type": event_type,
        "request_id": Uuid::new_v4(),
        "timestamp": Utc::now(),
        "payload": payload,
    })
    .to_string()
}

fn typing_key(conversation_id: Uuid, user_id: Uuid) -> String {
    format!("typing:{conversation_id}:{user_id}")
}

async fn ensure_member(
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
        return Err(AppError::Forbidden("Not a conversation member".into()));
    }
    Ok(())
}

async fn broadcast(
    state: &AppState,
    conversation_id: Uuid,
    sender_id: Uuid,
    frame: &str,
) -> Result<(), AppError> {
    let recipients = sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM conversation_members
         WHERE conversation_id=$1 AND left_at IS NULL AND user_id<>$2",
    )
    .bind(conversation_id)
    .bind(sender_id)
    .fetch_all(state.db())
    .await?;

    for recipient_id in recipients {
        let _ = state
            .connection_manager()
            .send_to_user(recipient_id, Message::Text(frame.to_owned()), None)
            .await;
        if let Err(error) = pubsub::publish_user_frame(state, recipient_id, frame).await {
            tracing::debug!(%error, %recipient_id, "cross-instance typing publish failed");
        }
    }
    Ok(())
}
