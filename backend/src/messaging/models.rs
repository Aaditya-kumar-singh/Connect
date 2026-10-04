use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub client_message_id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Option<Uuid>,
    pub content: Option<String>,
    pub content_type: String,
    pub reply_to_message_id: Option<Uuid>,
    pub forwarded_from_message_id: Option<Uuid>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessagePayload {
    pub conversation_id: Uuid,
    pub client_message_id: Uuid,
    pub content: String,
    pub content_type: String,
    pub reply_to_message_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct EditMessagePayload {
    pub message_id: Uuid,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteMessagePayload {
    pub message_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ReactionPayload {
    pub message_id: Uuid,
    pub emoji: String,
    pub action: String,
}

#[derive(Debug, Deserialize)]
pub struct MessageDeliveredPayload {
    pub message_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct MessageReadPayload {
    pub conversation_id: Uuid,
    pub last_read_message_id: Uuid,
}

#[derive(Debug, Serialize, Clone)]
pub struct MessageStatusPayload {
    pub message_id: Uuid,
    pub status: String,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct MessageHistoryQuery {
    pub limit: Option<u8>,
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ForwardMessageRequest {
    pub message_id: Uuid,
    pub conversation_id: Uuid,
    pub client_message_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct MessageHistory {
    pub items: Vec<Message>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MessageAckPayload {
    pub client_message_id: Uuid,
    pub server_message_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone)]
pub struct MessageAttachmentPayload {
    pub media_id: Uuid,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub media_type: String,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MessageNewPayload {
    pub message: Message,
    pub attachments: Vec<MessageAttachmentPayload>,
}

#[derive(Debug, Serialize)]
pub struct MessageEditedPayload {
    pub message_id: Uuid,
    pub content: String,
    pub edited_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MessageDeletedPayload {
    pub message_id: Uuid,
    pub deleted_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MessageReactionPayload {
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
    pub action: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorPayload {
    pub code: &'static str,
    pub message: String,
}
