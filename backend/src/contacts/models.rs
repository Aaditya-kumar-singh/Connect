use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Contact {
    pub user_id: Uuid,
    pub contact_user_id: Uuid,
    pub nickname: Option<String>,
    pub created_at: DateTime<Utc>,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddContactRequest {
    pub user_id: Uuid,
    pub nickname: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddBlockRequest {
    pub user_id: Uuid,
}
