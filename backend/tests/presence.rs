use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;
use ybm_connect::presence::{frame, PresenceUpdatePayload};

#[test]
fn presence_frame_has_expected_contract() {
    let user_id = Uuid::new_v4();
    let timestamp = Utc::now();
    let frame_value: Value = serde_json::from_str(&frame(PresenceUpdatePayload {
        user_id,
        status: "offline".into(),
        last_seen_at: Some(timestamp),
    }))
    .unwrap();

    assert_eq!(frame_value["type"], "presence.update");
    assert_eq!(frame_value["payload"]["user_id"], user_id.to_string());
    assert_eq!(frame_value["payload"]["status"], "offline");
    assert_eq!(
        frame_value["payload"]["last_seen_at"]
            .as_str()
            .and_then(|value| value.parse::<chrono::DateTime<Utc>>().ok()),
        Some(timestamp)
    );
}
