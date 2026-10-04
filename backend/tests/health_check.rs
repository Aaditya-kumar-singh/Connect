use axum::body::to_bytes;
use axum::http::Request;
use sqlx::postgres::PgPoolOptions;
use tower::util::ServiceExt;
use ybm_connect::{app_state::AppState, config::Config, router};

fn test_config() -> Config {
    Config {
        server_host: "127.0.0.1".to_string(),
        server_port: 8080,
        rust_log: "ybm_connect=info".to_string(),
        environment: "test".to_string(),
        database_url: "postgres://ybm:ybm_dev_password@127.0.0.1:1/ybm_connect".to_string(),
        database_max_connections: 5,
        database_min_connections: 1,
        redis_url: "redis://127.0.0.1:1".to_string(),
        redis_max_connections: 5,
        access_token_secret: "test-access-secret".to_string(),
        refresh_token_secret: "test-refresh-secret".to_string(),
        access_token_expiry_seconds: 900,
        refresh_token_expiry_days: 30,
        argon2_memory_kb: 19456,
        argon2_iterations: 2,
        argon2_parallelism: 1,
        email_provider: "console".to_string(),
        email_from: "noreply@test.local".to_string(),
        cors_allowed_origins: "http://localhost:3000".to_string(),
        trust_proxy_headers: false,
        r2_enabled: false,
        r2_access_key_id: "test-access-key".to_string(),
        r2_secret_access_key: "test-secret-key".to_string(),
        r2_bucket_name: "test-bucket".to_string(),
        r2_endpoint: "http://127.0.0.1:9000".to_string(),
        push_provider: "console".to_string(),
        fcm_credentials_path: None,
        fcm_credentials_json: None,
        vapid_private_key: None,
        vapid_public_key: None,
        vapid_subject: None,
        turn_server_url: None,
        turn_secret: None,
        turn_realm: None,
    }
}

fn test_state() -> AppState {
    let config = test_config();
    let db_pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect_lazy(&config.database_url)
        .expect("test database URL must be valid");
    let redis =
        redis::Client::open(config.redis_url.as_str()).expect("test Redis URL must be valid");
    AppState::new(config, db_pool, redis)
}

#[tokio::test]
async fn test_health_liveness_returns_200_without_dependencies() {
    let app = router::build(test_state());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    assert!(response.headers().contains_key("x-request-id"));
    assert_eq!(
        response.headers().get("x-content-type-options").unwrap(),
        "nosniff"
    );
    assert_eq!(response.headers().get("x-frame-options").unwrap(), "DENY");
    assert_eq!(
        response.headers().get("referrer-policy").unwrap(),
        "no-referrer"
    );

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
    assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
    assert!(json["timestamp"].is_string());
}

#[tokio::test]
async fn test_metrics_endpoint_returns_prometheus_text() {
    let app = router::build(test_state());
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        prometheus::TEXT_FORMAT
    );

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();
    assert!(text.contains("http_requests_total"));
    assert!(text.contains("http_request_duration_seconds"));
    assert!(text.contains("ws_connections_active"));
    assert!(text.contains("db_pool_connections_max"));
}

#[tokio::test]
async fn test_readiness_probe_returns_503_when_dependencies_are_unavailable() {
    let app = router::build(test_state());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/ready")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    );
    assert!(response.headers().contains_key("x-request-id"));

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "degraded");
    assert!(json["database"]["status"].is_string());
    assert!(json["redis"]["status"].is_string());
}

#[tokio::test]
#[ignore = "requires PostgreSQL and Redis configured through TEST_DATABASE_URL and TEST_REDIS_URL"]
async fn test_readiness_probe_returns_200_when_healthy() {
    let database_url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
    let redis_url = std::env::var("TEST_REDIS_URL").expect("TEST_REDIS_URL must be set");
    let mut config = test_config();
    config.database_url = database_url.clone();
    config.redis_url = redis_url.clone();
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&database_url)
        .unwrap();
    let redis = redis::Client::open(redis_url).unwrap();
    let state = AppState::new(config, db_pool, redis);
    let app = router::build(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/ready")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ready");
    assert_eq!(json["database"]["status"], "healthy");
    assert_eq!(json["redis"]["status"], "healthy");
}
