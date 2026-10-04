use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct ClientFrame {
    #[serde(rename = "type")]
    pub event_type: String,
    pub request_id: Option<Uuid>,
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerFrame<T: Serialize> {
    #[serde(rename = "type")]
    pub event_type: &'static str,
    pub request_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub payload: T,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectedPayload {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub device_id: Uuid,
    pub server_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmptyPayload {}

pub fn connected_frame(session_id: Uuid, user_id: Uuid, device_id: Uuid) -> String {
    serde_json::to_string(&ServerFrame {
        event_type: "server.connected",
        request_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        payload: ConnectedPayload {
            session_id,
            user_id,
            device_id,
            server_time: Utc::now(),
        },
    })
    .expect("server.connected payload is serializable")
}

pub fn pong_frame(request_id: Option<Uuid>) -> String {
    serde_json::to_string(&ServerFrame {
        event_type: "auth.pong",
        request_id: request_id.unwrap_or_else(Uuid::new_v4),
        timestamp: Utc::now(),
        payload: EmptyPayload {},
    })
    .expect("auth.pong payload is serializable")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connected_frame_has_protocol_type() {
        let value: serde_json::Value = serde_json::from_str(&connected_frame(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ))
        .unwrap();
        assert_eq!(value["type"], "server.connected");
        assert!(value["payload"]["session_id"].is_string());
    }

    #[test]
    fn pong_preserves_request_id() {
        let request_id = Uuid::new_v4();
        let value: serde_json::Value = serde_json::from_str(&pong_frame(Some(request_id))).unwrap();
        assert_eq!(value["type"], "auth.pong");
        assert_eq!(value["request_id"], request_id.to_string());
    }
}
