use crate::{
    app_state::AppState,
    auth::{email::ConsoleEmailSender, models::*, service},
    domain::errors::AppError,
};
use axum::{extract::State, Json};

pub async fn register(
    State(state): State<AppState>,
    Json(input): Json<RegisterRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    service::register(&state, input, &ConsoleEmailSender).await?;
    Ok(Json(MessageResponse {
        message: "Registration successful. Verify your email.",
    }))
}

pub async fn verify_email(
    State(state): State<AppState>,
    Json(input): Json<VerifyEmailRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    service::verify_email(&state, input).await?;
    Ok(Json(MessageResponse {
        message: "Email verified.",
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    Ok(Json(service::login(&state, input).await?))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(input): Json<RefreshRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    Ok(Json(service::refresh(&state, input).await?))
}

pub async fn logout(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
) -> Result<Json<MessageResponse>, AppError> {
    service::logout(&state, auth).await?;
    Ok(Json(MessageResponse {
        message: "Logged out.",
    }))
}

pub async fn logout_all(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
) -> Result<Json<MessageResponse>, AppError> {
    service::logout_all(&state, auth).await?;
    Ok(Json(MessageResponse {
        message: "Logged out from all devices.",
    }))
}

pub async fn forgot_password(
    State(state): State<AppState>,
    Json(input): Json<ForgotPasswordRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    service::forgot_password(&state, input, &ConsoleEmailSender).await?;
    Ok(Json(MessageResponse {
        message: "If the email exists, a reset code was sent.",
    }))
}

pub async fn reset_password(
    State(state): State<AppState>,
    Json(input): Json<ResetPasswordRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    service::reset_password(&state, input).await?;
    Ok(Json(MessageResponse {
        message: "Password reset. Please login.",
    }))
}
