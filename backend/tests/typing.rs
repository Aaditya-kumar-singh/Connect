use chrono::DateTime;
use serde_json::Value;
use uuid::Uuid;
use ybm_connect::typing::{frame, TypingUpdatePayload, TYPING_TTL_SECONDS};

#[test]
fn typing_ttl_matches_contract() {
    assert_eq!(TYPING_TTL_SECONDS, 5);
}

#[test]
fn typing_frame_matches_protocol() {
    let conversation_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let value: Value = serde_json::from_str(&frame(
        "typing.start",
        TypingUpdatePayload {
            conversation_id,
            user_id,
            display_name: "Alice".into(),
        },
    ))
    .expect("valid frame");

    assert_eq!(value["type"], "typing.start");
    assert_eq!(
        value["payload"]["conversation_id"],
        conversation_id.to_string()
    );
    assert_eq!(value["payload"]["user_id"], user_id.to_string());
    assert_eq!(value["payload"]["display_name"], "Alice");
    assert!(value["request_id"]
        .as_str()
        .and_then(|v| Uuid::parse_str(v).ok())
        .is_some());
    assert!(value["timestamp"]
        .as_str()
        .and_then(|v| DateTime::parse_from_rfc3339(v).ok())
        .is_some());
}

#[test]
fn typing_stop_uses_same_protocol_payload() {
    let value: Value = serde_json::from_str(&frame(
        "typing.stop",
        TypingUpdatePayload {
            conversation_id: Uuid::nil(),
            user_id: Uuid::nil(),
            display_name: "Alice".into(),
        },
    ))
    .expect("valid frame");
    assert_eq!(value["type"], "typing.stop");
    assert_eq!(value["payload"]["display_name"], "Alice");
}
