use crate::app_state::AppState;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::warn;

pub const EVENTS_CHANNEL: &str = "ybm:reliability:events";

#[derive(Debug, Serialize)]
struct ReliabilityEvent<'a> {
    event: &'a str,
    timestamp: u64,
    source: &'static str,
}

pub async fn publish(state: &AppState, event: &str) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();

    let payload = match serde_json::to_string(&ReliabilityEvent {
        event,
        timestamp,
        source: "rust",
    }) {
        Ok(payload) => payload,
        Err(error) => {
            warn!(%error, "Failed to serialize reliability event");
            return;
        }
    };

    let mut connection = match state.redis().get_multiplexed_async_connection().await {
        Ok(connection) => connection,
        Err(error) => {
            warn!(%error, event, "Reliability event Redis connection unavailable");
            return;
        }
    };

    let result: redis::RedisResult<i32> = redis::cmd("PUBLISH")
        .arg(EVENTS_CHANNEL)
        .arg(payload)
        .query_async(&mut connection)
        .await;

    if let Err(error) = result {
        warn!(%error, event, "Failed to publish reliability event");
    }
}
