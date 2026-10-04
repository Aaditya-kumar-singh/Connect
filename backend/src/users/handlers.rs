use crate::{
    app_state::AppState,
    auth::models::AuthenticatedSession,
    domain::errors::AppError,
    users::{models::*, service},
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

pub async fn me(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
) -> Result<Json<MeProfile>, AppError> {
    Ok(Json(service::get_me(&state, &auth).await?))
}
pub async fn update_me(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Json(input): Json<UpdateProfileRequest>,
) -> Result<Json<MeProfile>, AppError> {
    Ok(Json(service::update_me(&state, &auth, input).await?))
}
pub async fn public_profile(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Path(id): Path<Uuid>,
) -> Result<Json<PublicProfile>, AppError> {
    Ok(Json(service::get_public(&state, &auth, id).await?))
}
pub async fn search(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<UserSearchResult>>, AppError> {
    Ok(Json(service::search(&state, &auth, &query.q).await?))
}
pub async fn devices(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
) -> Result<Json<Vec<Device>>, AppError> {
    Ok(Json(service::list_devices(&state, &auth).await?))
}
pub async fn update_device(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateDeviceRequest>,
) -> Result<Json<Device>, AppError> {
    Ok(Json(
        service::update_device(&state, &auth, id, input).await?,
    ))
}
pub async fn delete_device(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    service::delete_device(&state, &auth, id).await?;
    Ok(Json(MessageResponse {
        message: "Device removed.",
    }))
}
pub async fn sessions(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
) -> Result<Json<Vec<Session>>, AppError> {
    Ok(Json(service::list_sessions(&state, &auth).await?))
}
pub async fn revoke_session(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    service::revoke_session(&state, &auth, id).await?;
    Ok(Json(MessageResponse {
        message: "Session revoked.",
    }))
}
pub async fn contacts(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
) -> Result<Json<Vec<crate::contacts::models::Contact>>, AppError> {
    Ok(Json(service::list_contacts(&state, &auth).await?))
}
pub async fn add_contact(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Json(input): Json<crate::contacts::models::AddContactRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    service::add_contact(&state, &auth, input.user_id, input.nickname).await?;
    Ok(Json(MessageResponse {
        message: "Contact added.",
    }))
}
pub async fn remove_contact(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    service::remove_contact(&state, &auth, id).await?;
    Ok(Json(MessageResponse {
        message: "Contact removed.",
    }))
}
pub async fn blocks(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
) -> Result<Json<Vec<Uuid>>, AppError> {
    Ok(Json(service::list_blocks(&state, &auth).await?))
}
pub async fn add_block(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Json(input): Json<crate::contacts::models::AddBlockRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    service::add_block(&state, &auth, input.user_id).await?;
    Ok(Json(MessageResponse {
        message: "User blocked.",
    }))
}
pub async fn remove_block(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    service::remove_block(&state, &auth, id).await?;
    Ok(Json(MessageResponse {
        message: "User unblocked.",
    }))
}

#[derive(Debug, serde::Serialize)]
pub struct MessageResponse {
    pub message: &'static str,
}
