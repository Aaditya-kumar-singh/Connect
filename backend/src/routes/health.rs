use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::app_state::AppState;
use crate::infrastructure::{database, redis_client};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: &'static str,
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadinessResponse {
    pub status: &'static str,
    pub database: ComponentHealth,
    pub redis: ComponentHealth,
}

pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        timestamp: Utc::now().to_rfc3339(),
    })
}

pub async fn ready_handler(State(state): State<AppState>) -> impl IntoResponse {
    let timeout = Duration::from_secs(2);

    let db_pool = state.db().clone();
    let db_check = tokio::time::timeout(
        timeout,
        async move { database::check_health(&db_pool).await },
    );

    let redis_client = state.redis().clone();
    let redis_check = tokio::time::timeout(timeout, async move {
        redis_client::check_health(&redis_client).await
    });

    let (db_res, redis_res) = tokio::join!(db_check, redis_check);
    let (database, db_ok) = component_from_db(db_res);
    let (redis, redis_ok) = component_from_redis(redis_res);
    let ready = db_ok && redis_ok;

    let body = ReadinessResponse {
        status: if ready { "ready" } else { "degraded" },
        database,
        redis,
    };

    let status = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(body))
}

fn component_from_db(
    result: Result<Result<Duration, sqlx::Error>, tokio::time::error::Elapsed>,
) -> (ComponentHealth, bool) {
    match result {
        Ok(Ok(duration)) => (
            ComponentHealth {
                status: "healthy",
                latency_ms: Some(duration.as_millis() as u64),
                error: None,
            },
            true,
        ),
        Ok(Err(error)) => (
            ComponentHealth {
                status: "unhealthy",
                latency_ms: None,
                error: Some(error.to_string()),
            },
            false,
        ),
        Err(_) => (
            ComponentHealth {
                status: "timeout",
                latency_ms: None,
                error: Some("Database health check timed out (> 2s)".to_string()),
            },
            false,
        ),
    }
}

fn component_from_redis(
    result: Result<Result<Duration, redis::RedisError>, tokio::time::error::Elapsed>,
) -> (ComponentHealth, bool) {
    match result {
        Ok(Ok(duration)) => (
            ComponentHealth {
                status: "healthy",
                latency_ms: Some(duration.as_millis() as u64),
                error: None,
            },
            true,
        ),
        Ok(Err(error)) => (
            ComponentHealth {
                status: "unhealthy",
                latency_ms: None,
                error: Some(error.to_string()),
            },
            false,
        ),
        Err(_) => (
            ComponentHealth {
                status: "timeout",
                latency_ms: None,
                error: Some("Redis health check timed out (> 2s)".to_string()),
            },
            false,
        ),
    }
}
