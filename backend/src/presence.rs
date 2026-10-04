use crate::app_state::AppState;
use crate::domain::errors::AppError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

const PRESENCE_TTL_SECONDS: usize = 60;

#[derive(Debug, Serialize, Clone)]
pub struct PresenceUpdatePayload {
    pub user_id: Uuid,
    pub status: String,
    pub last_seen_at: Option<DateTime<Utc>>,
}

pub async fn connect(state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    let mut conn = state.redis().get_multiplexed_async_connection().await?;
    let key = format!("presence:{user_id}");
    let created: Option<String> = redis::cmd("SET")
        .arg(&key)
        .arg("online")
        .arg("EX")
        .arg(PRESENCE_TTL_SECONDS)
        .arg("NX")
        .query_async(&mut conn)
        .await?;

    if created.is_some() {
        broadcast(
            state,
            PresenceUpdatePayload {
                user_id,
                status: "online".into(),
                last_seen_at: None,
            },
        )
        .await?;
    }

    Ok(())
}

pub async fn heartbeat(state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    let mut conn = state.redis().get_multiplexed_async_connection().await?;
    let key = format!("presence:{user_id}");
    let renewed: i64 = redis::cmd("EXPIRE")
        .arg(&key)
        .arg(PRESENCE_TTL_SECONDS)
        .query_async(&mut conn)
        .await?;

    if renewed == 0 {
        let _: () = redis::cmd("SET")
            .arg(&key)
            .arg("online")
            .arg("EX")
            .arg(PRESENCE_TTL_SECONDS)
            .query_async(&mut conn)
            .await?;
        broadcast(
            state,
            PresenceUpdatePayload {
                user_id,
                status: "online".into(),
                last_seen_at: None,
            },
        )
        .await?;
    }

    Ok(())
}

pub async fn disconnect(
    state: &AppState,
    user_id: Uuid,
    connection_member: &str,
) -> Result<(), AppError> {
    let mut conn = state.redis().get_multiplexed_async_connection().await?;
    let connections_key = format!("connections:{user_id}");
    let presence_key = format!("presence:{user_id}");

    let remaining: i64 = redis::Script::new(
        r#"
        redis.call('SREM', KEYS[1], ARGV[1])
        local remaining = redis.call('SCARD', KEYS[1])
        if remaining == 0 then
            redis.call('DEL', KEYS[2])
        end
        return remaining
        "#,
    )
    .key(&connections_key)
    .key(&presence_key)
    .arg(connection_member)
    .invoke_async(&mut conn)
    .await?;

    if remaining == 0 {
        let now = Utc::now();
        sqlx::query("UPDATE users SET last_seen_at=$2,updated_at=$2 WHERE id=$1")
            .bind(user_id)
            .bind(now)
            .execute(state.db())
            .await?;

        broadcast(
            state,
            PresenceUpdatePayload {
                user_id,
                status: "offline".into(),
                last_seen_at: Some(now),
            },
        )
        .await?;
    }

    Ok(())
}

pub fn frame(payload: PresenceUpdatePayload) -> String {
    serde_json::json!({
        "type": "presence.update",
        "request_id": Uuid::new_v4(),
        "timestamp": Utc::now(),
        "payload": payload,
    })
    .to_string()
}

async fn broadcast(state: &AppState, payload: PresenceUpdatePayload) -> Result<(), AppError> {
    let recipients = sqlx::query_scalar::<_, Uuid>(
        "SELECT DISTINCT recipient_id
         FROM (
             SELECT c.user_id AS recipient_id
             FROM contacts c
             WHERE c.contact_user_id=$1
             UNION
             SELECT c.contact_user_id AS recipient_id
             FROM contacts c
             WHERE c.user_id=$1
             UNION
             SELECT cm2.user_id AS recipient_id
             FROM conversation_members cm1
             JOIN conversation_members cm2
               ON cm2.conversation_id=cm1.conversation_id
              AND cm2.user_id<>cm1.user_id
              AND cm2.left_at IS NULL
             WHERE cm1.user_id=$1 AND cm1.left_at IS NULL
         ) recipients
         JOIN users u ON u.id=recipient_id
         WHERE recipient_id<>$1 AND u.status='ACTIVE'",
    )
    .bind(payload.user_id)
    .fetch_all(state.db())
    .await?;

    let event = frame(payload);
    for recipient_id in recipients {
        let _ = state
            .connection_manager()
            .send_to_user(
                recipient_id,
                axum::extract::ws::Message::Text(event.clone()),
                None,
            )
            .await;
        if let Err(error) =
            crate::websocket::pubsub::publish_presence_frame(state, recipient_id, &event).await
        {
            tracing::debug!(%error, %recipient_id, "cross-instance presence publish failed");
        }
    }

    Ok(())
}
