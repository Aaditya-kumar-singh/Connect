use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct OfferInput {
    pub callee_id: Uuid,
    pub call_type: String,
    pub sdp_offer: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnswerInput {
    pub call_id: Uuid,
    pub sdp_answer: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IceCandidateInput {
    pub call_id: Uuid,
    pub candidate: IceCandidate,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_m_line_index: Option<u16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RejectInput {
    pub call_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EndInput {
    pub call_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CallRecord {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub initiated_by: Uuid,
    pub call_type: String,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i32>,
    pub end_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CallHistoryItem {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub initiated_by: Uuid,
    pub call_type: String,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i32>,
    pub end_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub other_user_id: Uuid,
    pub other_display_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CallHistoryResponse {
    pub items: Vec<CallHistoryItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TurnCredentials {
    pub urls: Vec<String>,
    pub username: String,
    pub credential: String,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct IceConfigResponse {
    pub ice_servers: Vec<IceServer>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IceServer {
    pub urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}
