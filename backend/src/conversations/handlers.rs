use crate::{
    app_state::AppState,
    auth::models::AuthenticatedSession,
    conversations::{models::*, service},
    domain::errors::AppError,
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<u8>,
    pub cursor: Option<String>,
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Json(input): Json<CreateConversationRequest>,
) -> Result<Json<ConversationDetail>, AppError> {
    Ok(Json(service::create(&state, &auth, input.user_id).await?))
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Query(query): Query<ListQuery>,
) -> Result<Json<ConversationList>, AppError> {
    Ok(Json(
        service::list(&state, &auth, query.limit, query.cursor.as_deref()).await?,
    ))
}

pub async fn detail(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Path(id): Path<Uuid>,
) -> Result<Json<ConversationDetail>, AppError> {
    Ok(Json(service::detail(&state, &auth, id).await?))
}
