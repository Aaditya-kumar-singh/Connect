use std::sync::{Arc, Mutex};

use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Response,
};
use tower::ServiceExt;
use uuid::Uuid;

use ybm_connect::{
    app_state::AppState,
    auth::{
        email::EmailSender,
        models::{LoginRequest, RegisterRequest, VerifyEmailRequest},
        service,
    },
    config::Config,
    infrastructure::{database, redis_client},
    router,
};

#[derive(Clone, Default)]
struct TestEmail {
    otp: Arc<Mutex<Option<String>>>,
}

#[axum::async_trait]
impl EmailSender for TestEmail {
    async fn send_otp(
        &self,
        _: &str,
        otp: &str,
        _: &str,
    ) -> Result<(), ybm_connect::domain::errors::AppError> {
        *self.otp.lock().unwrap() = Some(otp.to_owned());
        Ok(())
    }
}

async fn setup_user(
    state: &AppState,
    email: &str,
    sender: &TestEmail,
) -> ybm_connect::auth::models::TokenResponse {
    service::register(
        state,
        RegisterRequest {
            email: email.to_owned(),
            password: "StrongPass1!".into(),
            display_name: "Phase Three".into(),
        },
        sender,
    )
    .await
    .unwrap();
    let otp = sender.otp.lock().unwrap().clone().unwrap();
    service::verify_email(
        state,
        VerifyEmailRequest {
            email: email.to_owned(),
            otp,
        },
    )
    .await
    .unwrap();
    service::login(
        state,
        LoginRequest {
            email: email.to_owned(),
            password: "StrongPass1!".into(),
            device_id: Uuid::new_v4(),
            device_name: Some("Integration Test".into()),
            device_type: "WEB".into(),
        },
    )
    .await
    .unwrap()
}

fn test_state() -> Option<AppState> {
    let db_url = std::env::var("TEST_DATABASE_URL").ok()?;
    let redis_url = std::env::var("TEST_REDIS_URL").ok()?;
    let mut config = Config::from_env().ok()?;
    config.database_url = db_url.clone();
    config.redis_url = redis_url.clone();
    Some(AppState::new(
        config,
        database::create_pool(&db_url, 5, 1).ok()?,
        redis_client::create_client(&redis_url).ok()?,
    ))
}

async fn body_json(response: Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
#[ignore = "requires PostgreSQL and Redis configured through TEST_DATABASE_URL and TEST_REDIS_URL"]
async fn profile_contacts_and_blocks_http_flow() {
    let state = test_state().unwrap();
    sqlx::migrate!("./migrations")
        .run(state.db())
        .await
        .unwrap();

    let sender = TestEmail::default();
    let token = setup_user(
        &state,
        &format!("phase3-{}@example.com", Uuid::new_v4()),
        &sender,
    )
    .await;

    let app = router::build(state);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/users/me")
                .header("authorization", format!("Bearer {}", token.access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let me = body_json(response).await;
    assert_eq!(me["display_name"], "Phase Three");

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/contacts")
                .header("authorization", format!("Bearer {}", token.access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
