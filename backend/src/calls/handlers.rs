use crate::{
    app_state::AppState,
    auth::models::AuthenticatedSession,
    calls::{models::*, service},
    domain::errors::AppError,
};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<u8>,
}

pub async fn history(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<CallHistoryResponse>, AppError> {
    Ok(Json(service::history(&state, &auth, query.limit).await?))
}

pub async fn ice_config(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
) -> Result<Json<IceConfigResponse>, AppError> {
    Ok(Json(service::ice_config(&state, &auth).await?))
}
