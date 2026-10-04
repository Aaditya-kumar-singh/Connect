use crate::{
    app_state::AppState,
    auth::{models::AuthenticatedSession, service},
    domain::errors::AppError,
};
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthenticatedSession {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Authorization required".into()))?;
        let token = value
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Authorization required".into()))?;
        service::authenticate(state, token).await
    }
}
