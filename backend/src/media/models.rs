use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct MediaObject {
    pub id: Uuid,
    pub uploader_id: Uuid,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub media_type: String,
    pub r2_key: String,
    pub thumbnail_r2_key: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_seconds: Option<f32>,
    pub checksum_sha256: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub media_id: Uuid,
    pub message_id: Uuid,
    pub media_type: String,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SignedMediaUrl {
    pub url: String,
    pub expires_in: u64,
}
