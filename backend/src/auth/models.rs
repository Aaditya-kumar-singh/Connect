use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct RefreshTokenRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub token_hash: String,
    pub family_id: Uuid,
    pub revoked_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedSession {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub device_id: Uuid,
    pub access_token_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
}
#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub email: String,
    pub otp: String,
}
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub device_id: Uuid,
    pub device_name: Option<String>,
    pub device_type: String,
}
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}
#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}
#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub otp: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_in: u64,
    pub refresh_token_expires_in: u64,
    pub session_id: Uuid,
    pub device_id: Uuid,
}
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: &'static str,
}
