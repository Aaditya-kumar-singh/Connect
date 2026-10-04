use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MeProfile {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub status: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub phone_number: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PublicProfile {
    pub id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub phone_number: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserSearchResult {
    pub id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Device {
    pub id: Uuid,
    pub device_name: Option<String>,
    pub device_type: String,
    pub push_token: Option<String>,
    pub push_provider: Option<String>,
    pub last_active_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDeviceRequest {
    pub push_token: Option<String>,
    pub push_provider: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub device_id: Uuid,
    pub device_name: Option<String>,
    pub device_type: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
