pub mod health;

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    match state.metrics().render(state.db()) {
        Ok(body) => ([(header::CONTENT_TYPE, prometheus::TEXT_FORMAT)], body).into_response(),
        Err(error) => {
            tracing::error!(%error, "failed to encode Prometheus metrics");
            (StatusCode::INTERNAL_SERVER_ERROR, "metrics unavailable").into_response()
        }
    }
}

async fn security_headers_middleware(request: Request, next: middleware::Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    headers.insert(
        "content-security-policy",
        HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'"),
    );
    response
}

async fn metrics_middleware(
    State(state): State<AppState>,
    request: Request,
    next: middleware::Next,
) -> Response {
    let method = request.method().as_str().to_owned();
    let path = request.uri().path().to_owned();
    let started = std::time::Instant::now();
    let response = next.run(request).await;
    state.metrics().observe_http(
        &method,
        &path,
        response.status().as_u16(),
        started.elapsed(),
    );
    response
}

use crate::{app_state::AppState, auth::rate_limit};
use axum::{
    extract::{DefaultBodyLimit, Request, State},
    http::{header, HeaderName, HeaderValue, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use std::time::Duration;
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

pub fn create_router(state: AppState) -> Router {
    let request_id_header = HeaderName::from_static("x-request-id");
    let origins = state
        .config()
        .cors_allowed_origins
        .split(',')
        .filter_map(|origin| origin.trim().parse::<HeaderValue>().ok())
        .collect::<Vec<_>>();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            http::Method::GET,
            http::Method::POST,
            http::Method::PATCH,
            http::Method::DELETE,
            http::Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            HeaderName::from_static("x-device-id"),
            HeaderName::from_static("x-request-id"),
        ]);

    Router::new()
        .route("/health", get(health::health_handler))
        .route("/ready", get(health::ready_handler))
        .route("/metrics", get(metrics_handler))
        .route(
            "/api/v1/auth/register",
            post(crate::auth::handlers::register),
        )
        .route(
            "/api/v1/auth/verify-email",
            post(crate::auth::handlers::verify_email),
        )
        .route("/api/v1/auth/login", post(crate::auth::handlers::login))
        .route("/api/v1/auth/refresh", post(crate::auth::handlers::refresh))
        .route("/api/v1/auth/logout", post(crate::auth::handlers::logout))
        .route(
            "/api/v1/auth/logout-all",
            post(crate::auth::handlers::logout_all),
        )
        .route(
            "/api/v1/auth/forgot-password",
            post(crate::auth::handlers::forgot_password),
        )
        .route(
            "/api/v1/auth/reset-password",
            post(crate::auth::handlers::reset_password),
        )
        .route(
            "/api/v1/users/me",
            get(crate::users::handlers::me).patch(crate::users::handlers::update_me),
        )
        .route("/api/v1/users/search", get(crate::users::handlers::search))
        .route(
            "/api/v1/users/{id}",
            get(crate::users::handlers::public_profile),
        )
        .route("/api/v1/devices", get(crate::users::handlers::devices))
        .route(
            "/api/v1/devices/{id}",
            axum::routing::patch(crate::users::handlers::update_device)
                .delete(crate::users::handlers::delete_device),
        )
        .route("/api/v1/sessions", get(crate::users::handlers::sessions))
        .route(
            "/api/v1/sessions/{id}",
            axum::routing::delete(crate::users::handlers::revoke_session),
        )
        .route(
            "/api/v1/contacts",
            get(crate::users::handlers::contacts).post(crate::users::handlers::add_contact),
        )
        .route(
            "/api/v1/contacts/{id}",
            axum::routing::delete(crate::users::handlers::remove_contact),
        )
        .route(
            "/api/v1/blocks",
            get(crate::users::handlers::blocks).post(crate::users::handlers::add_block),
        )
        .route(
            "/api/v1/blocks/{id}",
            axum::routing::delete(crate::users::handlers::remove_block),
        )
        .route(
            "/api/v1/conversations",
            get(crate::conversations::handlers::list).post(crate::conversations::handlers::create),
        )
        .route(
            "/api/v1/conversations/{id}",
            get(crate::conversations::handlers::detail),
        )
        .route(
            "/api/v1/conversations/{id}/messages",
            get(crate::messaging::handlers::history),
        )
        .route(
            "/api/v1/messages/forward",
            post(crate::messaging::handlers::forward),
        )
        .route(
            "/api/v1/media/upload",
            post(crate::media::handlers::upload).layer(DefaultBodyLimit::max(51 * 1024 * 1024)),
        )
        .route(
            "/api/v1/calls/history",
            get(crate::calls::handlers::history),
        )
        .route(
            "/api/v1/calls/ice-config",
            get(crate::calls::handlers::ice_config),
        )
        .route(
            "/api/v1/media/{id}/url",
            get(crate::media::handlers::download_url),
        )
        .route("/api/v1/groups", post(crate::groups::handlers::create))
        .route(
            "/api/v1/groups/{id}",
            get(crate::groups::handlers::detail)
                .patch(crate::groups::handlers::update)
                .delete(crate::groups::handlers::delete),
        )
        .route(
            "/api/v1/groups/{id}/members",
            post(crate::groups::handlers::add_members),
        )
        .route(
            "/api/v1/groups/{id}/members/{user_id}",
            axum::routing::delete(crate::groups::handlers::remove_member),
        )
        .route(
            "/api/v1/groups/{id}/members/{user_id}/role",
            axum::routing::patch(crate::groups::handlers::change_role),
        )
        .route(
            "/api/v1/groups/{id}/leave",
            post(crate::groups::handlers::leave),
        )
        .route("/ws/connect", get(crate::websocket::handlers::connect))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit::middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            metrics_middleware,
        ))
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        .layer(PropagateRequestIdLayer::new(request_id_header.clone()))
        .layer(SetRequestIdLayer::new(request_id_header, MakeRequestUuid))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request| {
                let request_id = request
                    .headers()
                    .get("x-request-id")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or("missing");
                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    path = %request.uri().path(),
                    request_id = %request_id,
                )
            }),
        )
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .with_state(state)
}
