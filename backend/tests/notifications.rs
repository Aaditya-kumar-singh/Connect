use ybm_connect::notifications::models::{NotificationPayload, PushErrorKind};
use ybm_connect::notifications::provider::{ConsolePushProvider, PushProvider};

#[tokio::test]
async fn console_provider_accepts_notification_without_logging_secret() {
    let provider = ConsolePushProvider;
    let payload = NotificationPayload {
        notification_type: "NEW_MESSAGE".into(),
        title: "Alice".into(),
        body: Some("Hello".into()),
        data: serde_json::json!({"conversation_id":"test"}),
        tag: Some("conversation:test".into()),
    };
    assert!(provider.send("secret-token", &payload).await.is_ok());
}

#[test]
fn invalid_token_kind_is_distinct() {
    let error = ybm_connect::notifications::models::PushError::Provider {
        kind: PushErrorKind::InvalidToken,
        message: "invalid".into(),
    };
    assert_eq!(error.kind(), PushErrorKind::InvalidToken);
}
