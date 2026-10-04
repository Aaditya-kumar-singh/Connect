use uuid::Uuid;
use ybm_connect::conversations::validation::{decode_cursor, encode_cursor, validate_limit};

#[test]
fn validates_conversation_limits() {
    assert_eq!(validate_limit(None).unwrap(), 20);
    assert_eq!(validate_limit(Some(50)).unwrap(), 50);
    assert!(validate_limit(Some(0)).is_err());
    assert!(validate_limit(Some(51)).is_err());
}

#[test]
fn cursor_round_trip() {
    let id = Uuid::new_v4();
    let timestamp = "2026-10-03T12:00:00+00:00";
    let cursor = encode_cursor(timestamp, id);
    let decoded = decode_cursor(Some(&cursor)).unwrap().unwrap();
    assert_eq!(decoded.0, timestamp);
    assert_eq!(decoded.1, id);
}

#[test]
fn rejects_invalid_cursor() {
    assert!(decode_cursor(Some("not-a-cursor")).is_err());
}
