use std::sync::{Arc, Mutex};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use uuid::Uuid;

use ybm_connect::{
    app_state::AppState,
    auth::{
        email::EmailSender,
        models::{
            ForgotPasswordRequest, LoginRequest, RefreshRequest, RegisterRequest,
            ResetPasswordRequest, VerifyEmailRequest,
        },
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
        _recipient: &str,
        otp: &str,
        _purpose: &str,
    ) -> Result<(), ybm_connect::domain::errors::AppError> {
        *self.otp.lock().unwrap() = Some(otp.to_string());
        Ok(())
    }
}

fn test_state() -> Option<AppState> {
    let db_url = std::env::var("TEST_DATABASE_URL").ok()?;
    let redis_url = std::env::var("TEST_REDIS_URL").ok()?;
    let mut config = Config::from_env().ok()?;
    config.database_url = db_url.clone();
    config.redis_url = redis_url.clone();
    let db = database::create_pool(&db_url, 5, 1).ok()?;
    let redis = redis_client::create_client(&redis_url).ok()?;
    Some(AppState::new(config, db, redis))
}

#[tokio::test]
#[ignore = "requires PostgreSQL and Redis configured through TEST_DATABASE_URL and TEST_REDIS_URL"]
async fn authentication_flow_covers_registration_login_refresh_logout_and_reset() {
    let state = test_state().expect("test services");
    sqlx::migrate!("./migrations")
        .run(state.db())
        .await
        .expect("migrations");
    let email = format!("auth-test-{}@example.com", Uuid::new_v4());
    let sender = TestEmail::default();

    service::register(
        &state,
        RegisterRequest {
            email: email.clone(),
            password: "StrongPass1!".into(),
            display_name: "Auth Test".into(),
        },
        &sender,
    )
    .await
    .expect("register");

    let verification_otp = sender
        .otp
        .lock()
        .unwrap()
        .clone()
        .expect("verification otp");
    service::verify_email(
        &state,
        VerifyEmailRequest {
            email: email.clone(),
            otp: verification_otp,
        },
    )
    .await
    .expect("verify");

    let device_id = Uuid::new_v4();
    let tokens = service::login(
        &state,
        LoginRequest {
            email: email.clone(),
            password: "StrongPass1!".into(),
            device_id,
            device_name: Some("Test Browser".into()),
            device_type: "WEB".into(),
        },
    )
    .await
    .expect("login");

    let session = service::authenticate(&state, &tokens.access_token)
        .await
        .expect("authenticate");
    let refreshed = service::refresh(
        &state,
        RefreshRequest {
            refresh_token: tokens.refresh_token,
        },
    )
    .await
    .expect("refresh");

    assert_ne!(refreshed.access_token, tokens.access_token);
    service::logout(&state, session).await.expect("logout");
    assert!(service::authenticate(&state, &tokens.access_token)
        .await
        .is_err());

    let reset_sender = TestEmail::default();
    service::forgot_password(
        &state,
        ForgotPasswordRequest {
            email: email.clone(),
        },
        &reset_sender,
    )
    .await
    .expect("forgot password");
    let reset_otp = reset_sender.otp.lock().unwrap().clone().expect("reset otp");
    service::reset_password(
        &state,
        ResetPasswordRequest {
            email,
            otp: reset_otp,
            new_password: "NewStrongPass1!".into(),
        },
    )
    .await
    .expect("reset password");

    let app = router::build(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
