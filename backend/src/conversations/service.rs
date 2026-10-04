use crate::{
    app_state::AppState,
    auth::models::AuthenticatedSession,
    contacts::repository as contact_repo,
    conversations::{models::*, repository, validation},
    domain::errors::AppError,
};
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub async fn create(
    state: &AppState,
    auth: &AuthenticatedSession,
    other_user_id: Uuid,
) -> Result<ConversationDetail, AppError> {
    if other_user_id == auth.user_id {
        return Err(AppError::Validation(
            "Cannot create a conversation with yourself".into(),
        ));
    }
    if !contact_repo::user_exists(state.db(), other_user_id).await? {
        return Err(AppError::NotFound("User not found".into()));
    }
    if contact_repo::is_blocked(state.db(), auth.user_id, other_user_id).await?
        || contact_repo::is_blocked(state.db(), other_user_id, auth.user_id).await?
    {
        return Err(AppError::Forbidden("Conversation is blocked".into()));
    }
    let id = repository::create_direct(state.db(), auth.user_id, other_user_id).await?;
    repository::get_direct(state.db(), auth.user_id, id).await
}

pub async fn list(
    state: &AppState,
    auth: &AuthenticatedSession,
    limit: Option<u8>,
    cursor: Option<&str>,
) -> Result<ConversationList, AppError> {
    let limit = validation::validate_limit(limit)?;
    let decoded = validation::decode_cursor(cursor)?;
    let decoded = decoded
        .map(|(ts, id)| {
            DateTime::parse_from_rfc3339(&ts)
                .map(|v| (v.with_timezone(&Utc), id))
                .map_err(|_| AppError::Validation("Invalid cursor".into()))
        })
        .transpose()?;
    let rows = repository::list_direct(state.db(), auth.user_id, limit + 1, decoded).await?;
    let has_more = rows.len() > limit as usize;
    let mut items = rows;
    if has_more {
        items.pop();
    }
    let next_cursor = items
        .last()
        .map(|row| validation::encode_cursor(&row.updated_at.to_rfc3339(), row.id));
    Ok(ConversationList {
        items,
        next_cursor: if has_more { next_cursor } else { None },
    })
}

pub async fn detail(
    state: &AppState,
    auth: &AuthenticatedSession,
    id: Uuid,
) -> Result<ConversationDetail, AppError> {
    repository::get_direct(state.db(), auth.user_id, id).await
}
