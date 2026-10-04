use crate::{
    app_state::AppState,
    auth::{models::AuthenticatedSession, service as auth_service},
    contacts::{models::Contact, repository as contact_repo},
    domain::errors::AppError,
    users::{models::*, repository, validation},
};
use uuid::Uuid;

pub async fn get_me(state: &AppState, auth: &AuthenticatedSession) -> Result<MeProfile, AppError> {
    repository::ensure_profile(state.db(), auth.user_id).await?;
    repository::get_me(state.db(), auth.user_id).await
}

pub async fn update_me(
    state: &AppState,
    auth: &AuthenticatedSession,
    input: UpdateProfileRequest,
) -> Result<MeProfile, AppError> {
    validation::validate_profile(&input)?;
    if let Some(name) = &input.display_name {
        if name.trim().is_empty() {
            return Err(AppError::Validation("Display name cannot be empty".into()));
        }
    }
    repository::update_me(state.db(), auth.user_id, &input).await
}

pub async fn get_public(
    state: &AppState,
    _auth: &AuthenticatedSession,
    user_id: Uuid,
) -> Result<PublicProfile, AppError> {
    repository::get_public(state.db(), user_id).await
}

pub async fn search(
    state: &AppState,
    _auth: &AuthenticatedSession,
    query: &str,
) -> Result<Vec<UserSearchResult>, AppError> {
    let query = query.trim();
    if query.len() < 2 {
        return Err(AppError::Validation(
            "Search query must be at least 2 characters".into(),
        ));
    }
    if query.len() > 100 {
        return Err(AppError::Validation(
            "Search query must be at most 100 characters".into(),
        ));
    }
    repository::search(state.db(), query, 20).await
}

pub async fn list_devices(
    state: &AppState,
    auth: &AuthenticatedSession,
) -> Result<Vec<Device>, AppError> {
    repository::list_devices(state.db(), auth.user_id).await
}

pub async fn update_device(
    state: &AppState,
    auth: &AuthenticatedSession,
    device_id: Uuid,
    input: UpdateDeviceRequest,
) -> Result<Device, AppError> {
    validation::validate_device_update(&input)?;
    repository::update_device(state.db(), auth.user_id, device_id, &input).await
}

pub async fn delete_device(
    state: &AppState,
    auth: &AuthenticatedSession,
    device_id: Uuid,
) -> Result<(), AppError> {
    let sessions = repository::delete_device(state.db(), auth.user_id, device_id).await?;
    for session_id in sessions {
        auth_service::revoke_session_by_id(state, session_id).await?;
    }
    Ok(())
}

pub async fn list_sessions(
    state: &AppState,
    auth: &AuthenticatedSession,
) -> Result<Vec<Session>, AppError> {
    repository::list_sessions(state.db(), auth.user_id).await
}

pub async fn revoke_session(
    state: &AppState,
    auth: &AuthenticatedSession,
    session_id: Uuid,
) -> Result<(), AppError> {
    let sessions = repository::list_sessions(state.db(), auth.user_id).await?;
    if !sessions.iter().any(|s| s.id == session_id) {
        return Err(AppError::NotFound("Session not found".into()));
    }
    auth_service::revoke_session_by_id(state, session_id).await
}

pub async fn add_contact(
    state: &AppState,
    auth: &AuthenticatedSession,
    user_id: Uuid,
    nickname: Option<String>,
) -> Result<(), AppError> {
    if user_id == auth.user_id {
        return Err(AppError::Validation(
            "Cannot add yourself as a contact".into(),
        ));
    }
    if !contact_repo::user_exists(state.db(), user_id).await? {
        return Err(AppError::NotFound("User not found".into()));
    }
    if contact_repo::is_blocked(state.db(), auth.user_id, user_id).await?
        || contact_repo::is_blocked(state.db(), user_id, auth.user_id).await?
    {
        return Err(AppError::Forbidden("Contact is blocked".into()));
    }
    if nickname.as_ref().is_some_and(|v| v.chars().count() > 50) {
        return Err(AppError::Validation(
            "Nickname must be at most 50 characters".into(),
        ));
    }
    contact_repo::add_contact(state.db(), auth.user_id, user_id, nickname.as_deref()).await
}

pub async fn remove_contact(
    state: &AppState,
    auth: &AuthenticatedSession,
    user_id: Uuid,
) -> Result<(), AppError> {
    contact_repo::remove_contact(state.db(), auth.user_id, user_id).await
}

pub async fn list_contacts(
    state: &AppState,
    auth: &AuthenticatedSession,
) -> Result<Vec<Contact>, AppError> {
    contact_repo::list_contacts(state.db(), auth.user_id).await
}

pub async fn add_block(
    state: &AppState,
    auth: &AuthenticatedSession,
    user_id: Uuid,
) -> Result<(), AppError> {
    if user_id == auth.user_id {
        return Err(AppError::Validation("Cannot block yourself".into()));
    }
    if !contact_repo::user_exists(state.db(), user_id).await? {
        return Err(AppError::NotFound("User not found".into()));
    }
    contact_repo::add_block(state.db(), auth.user_id, user_id).await
}

pub async fn remove_block(
    state: &AppState,
    auth: &AuthenticatedSession,
    user_id: Uuid,
) -> Result<(), AppError> {
    contact_repo::remove_block(state.db(), auth.user_id, user_id).await
}

pub async fn list_blocks(
    state: &AppState,
    auth: &AuthenticatedSession,
) -> Result<Vec<Uuid>, AppError> {
    contact_repo::list_blocks(state.db(), auth.user_id).await
}
