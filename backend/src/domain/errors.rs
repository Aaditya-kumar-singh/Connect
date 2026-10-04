use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Rate limited")]
    RateLimited,
    #[error("Gone: {0}")]
    Gone(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AppError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            AppError::Forbidden(_) => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            AppError::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT"),
            AppError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, "VALIDATION_ERROR"),
            AppError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED"),
            AppError::Gone(_) => (StatusCode::GONE, "GONE"),
            AppError::ServiceUnavailable(_) => {
                (StatusCode::SERVICE_UNAVAILABLE, "SERVICE_UNAVAILABLE")
            }
            AppError::Internal(_) | AppError::Database(_) | AppError::Redis(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR")
            }
        };

        let message = match &self {
            AppError::Unauthorized(message) => message.clone(),
            AppError::Forbidden(message) => message.clone(),
            AppError::NotFound(message) => message.clone(),
            AppError::Conflict(message) => message.clone(),
            AppError::Validation(message) => message.clone(),
            AppError::Gone(message) => message.clone(),
            AppError::ServiceUnavailable(message) => message.clone(),
            AppError::RateLimited => "Too many requests".into(),
            AppError::Internal(_) | AppError::Database(_) | AppError::Redis(_) => {
                "Internal server error".into()
            }
        };

        let body = json!({
            "success": false,
            "data": null,
            "error": { "code": code, "message": message }
        });
        (status, Json(body)).into_response()
    }
}
