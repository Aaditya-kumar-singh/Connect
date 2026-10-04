use crate::app_state::AppState;
use axum::extract::ws::Message;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::oneshot;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct UserFrame {
    origin_instance_id: Uuid,
    user_id: Uuid,
    frame: String,
}

pub async fn publish_presence_frame(
    state: &AppState,
    user_id: Uuid,
    frame: &str,
) -> Result<(), redis::RedisError> {
    let payload = serde_json::to_string(&UserFrame {
        origin_instance_id: state.instance_id(),
        user_id,
        frame: frame.to_owned(),
    })
    .expect("presence frame is serializable");
    let mut conn = state.redis().get_multiplexed_async_connection().await?;
    let channel = format!("presence:user:{user_id}");
    let _: i64 = redis::cmd("PUBLISH")
        .arg(channel)
        .arg(payload)
        .query_async(&mut conn)
        .await?;
    Ok(())
}

pub async fn publish_user_frame(
    state: &AppState,
    user_id: Uuid,
    frame: &str,
) -> Result<(), redis::RedisError> {
    let payload = serde_json::to_string(&UserFrame {
        origin_instance_id: state.instance_id(),
        user_id,
        frame: frame.to_owned(),
    })
    .expect("user frame is serializable");
    let mut conn = state.redis().get_multiplexed_async_connection().await?;
    let channel = format!("messaging:user:{user_id}");
    let _: i64 = redis::cmd("PUBLISH")
        .arg(channel)
        .arg(payload)
        .query_async(&mut conn)
        .await?;
    Ok(())
}

pub async fn run(state: AppState, mut shutdown: oneshot::Receiver<()>) {
    let patterns = ["messaging:user:*", "presence:user:*"];
    let mut backoff = Duration::from_secs(1);

    loop {
        let client = state.redis().clone();
        match client.get_async_pubsub().await {
            Ok(mut pubsub) => {
                let subscribe_result = async {
                    for pattern in patterns {
                        pubsub.psubscribe(pattern).await?;
                    }
                    Ok::<(), redis::RedisError>(())
                }
                .await;
                if let Err(error) = subscribe_result {
                    tracing::warn!(%error, "Redis WebSocket pub/sub pattern subscribe failed");
                } else {
                    tracing::info!(patterns = ?patterns, "Redis WebSocket pub/sub subscriber started");
                    backoff = Duration::from_secs(1);
                    let mut messages = pubsub.on_message();
                    loop {
                        tokio::select! {
                            _ = &mut shutdown => return,
                            message = messages.next() => {
                                let Some(message) = message else { break; };
                                match message.get_payload::<String>() {
                                    Ok(payload) => match serde_json::from_str::<UserFrame>(&payload) {
                                        Ok(event) if event.origin_instance_id != state.instance_id() => {
                                            let _ = state.connection_manager()
                                                .send_to_user(event.user_id, Message::Text(event.frame), None)
                                                .await;
                                        }
                                        Ok(_) => {}
                                        Err(error) => tracing::debug!(%error, "invalid messaging pub/sub payload"),
                                    },
                                    Err(error) => tracing::debug!(%error, "invalid messaging pub/sub message payload"),
                                }
                            }
                        }
                    }
                }
            }
            Err(error) => {
                tracing::warn!(%error, "Redis WebSocket pub/sub connection failed");
            }
        }

        tokio::select! {
            _ = &mut shutdown => return,
            _ = tokio::time::sleep(backoff) => {}
        }
        backoff = (backoff * 2).min(Duration::from_secs(30));
    }
}
