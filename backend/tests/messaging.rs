use uuid::Uuid;
use ybm_connect::messaging::validation;

#[test]
fn validates_text_limits() {
    assert!(validation::validate_text("hello").is_ok());
    assert!(validation::validate_text("").is_err());
    assert!(validation::validate_text(&"x".repeat(4097)).is_err());
}

#[test]
fn validates_phase_six_content_type() {
    assert!(validation::validate_content_type("text").is_ok());
    assert!(validation::validate_content_type("image").is_err());
}

#[test]
fn cursor_round_trip() {
    let id = Uuid::new_v4();
    let cursor = validation::encode_cursor("2026-10-03T12:00:00Z", id);
    let decoded = validation::decode_cursor(Some(&cursor)).unwrap().unwrap();
    assert_eq!(decoded.0, "2026-10-03T12:00:00Z");
    assert_eq!(decoded.1, id);
}

#[test]
fn validates_reactions() {
    assert!(validation::validate_emoji("👍").is_ok());
    assert!(validation::validate_reaction_action("add").is_ok());
    assert!(validation::validate_reaction_action("remove").is_ok());
    assert!(validation::validate_reaction_action("replace").is_err());
}

#[test]
fn validates_delivery_batch() {
    assert!(validation::validate_message_ids(&[Uuid::new_v4()]).is_ok());
    assert!(validation::validate_message_ids(&[]).is_err());
}
