use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ConversationListQuery {
    pub limit: Option<u8>,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow, Clone)]
pub struct Conversation {
    pub id: Uuid,
    pub conversation_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow, Clone)]
pub struct DirectConversation {
    pub id: Uuid,
    pub conversation_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub other_user_id: Uuid,
    pub other_display_name: String,
    pub other_avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConversationList {
    pub items: Vec<DirectConversation>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConversationDetail {
    pub conversation: DirectConversation,
    pub member_ids: Vec<Uuid>,
}
