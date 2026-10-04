use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub notification_type: String,
    pub title: String,
    pub body: Option<String>,
    pub data: Value,
    pub tag: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NotificationJob {
    pub user_id: Uuid,
    pub notification_id: Uuid,
    pub payload: NotificationPayload,
}

#[derive(Debug, sqlx::FromRow)]
pub struct NotificationRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: String,
    pub title: String,
    pub body: Option<String>,
    pub data: Option<Value>,
    pub sent_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushErrorKind {
    Temporary,
    InvalidToken,
    Permanent,
}

#[derive(Debug, thiserror::Error)]
pub enum PushError {
    #[error("{message}")]
    Provider {
        kind: PushErrorKind,
        message: String,
    },
}

impl PushError {
    pub fn kind(&self) -> PushErrorKind {
        match self {
            Self::Provider { kind, .. } => *kind,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DevicePushTarget {
    pub device_id: Uuid,
    pub token: String,
    pub provider: String,
}
