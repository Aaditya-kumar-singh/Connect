use crate::{
    app_state::AppState,
    messaging::models::Message,
    notifications::{
        models::{NotificationJob, NotificationPayload},
        repository,
    },
};
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

pub async fn enqueue_new_message(state: &AppState, message: &Message) {
    let sender_id = match message.sender_id {
        Some(id) => id,
        None => return,
    };

    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            String,
            Option<String>,
            Option<chrono::DateTime<Utc>>,
        ),
    >(
        "SELECT cm.user_id, u.display_name, c.conversation_type,
                g.name, cm.muted_until
         FROM conversation_members cm
         JOIN users u ON u.id=cm.user_id
         JOIN conversations c ON c.id=cm.conversation_id
         LEFT JOIN groups g ON g.conversation_id=c.id AND g.deleted_at IS NULL
         WHERE cm.conversation_id=$1
           AND cm.user_id<>$2
           AND cm.left_at IS NULL
           AND u.deleted_at IS NULL",
    )
    .bind(message.conversation_id)
    .bind(sender_id)
    .fetch_all(state.db())
    .await;

    let rows = match rows {
        Ok(rows) => rows,
        Err(error) => {
            tracing::warn!(error = %error, message_id = %message.id, "notification recipient lookup failed");
            return;
        }
    };

    let sender_name = sqlx::query_scalar::<_, String>("SELECT display_name FROM users WHERE id=$1")
        .bind(sender_id)
        .fetch_optional(state.db())
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "New message".into());

    let body = message
        .content
        .as_deref()
        .unwrap_or("[media]")
        .chars()
        .take(100)
        .collect::<String>();

    for (user_id, _display_name, conversation_type, group_name, muted_until) in rows {
        if muted_until.is_some_and(|until| until > Utc::now()) {
            continue;
        }

        let mut redis = match state.redis().get_multiplexed_async_connection().await {
            Ok(redis) => redis,
            Err(error) => {
                tracing::warn!(error = %error, user_id = %user_id, "notification presence check failed");
                continue;
            }
        };
        let connection_key = format!("connections:{user_id}");
        let active_connections: i64 = match redis::cmd("SCARD")
            .arg(connection_key)
            .query_async(&mut redis)
            .await
        {
            Ok(count) => count,
            Err(error) => {
                tracing::warn!(error = %error, user_id = %user_id, "notification connection check failed");
                continue;
            }
        };
        if active_connections > 0 {
            continue;
        }

        let title = if conversation_type == "GROUP" {
            group_name.unwrap_or_else(|| sender_name.clone())
        } else {
            sender_name.clone()
        };
        let payload = NotificationPayload {
            notification_type: "NEW_MESSAGE".into(),
            title,
            body: Some(body.clone()),
            data: json!({
                "type": "NEW_MESSAGE",
                "conversation_id": message.conversation_id,
                "message_id": message.id,
                "sender_id": sender_id,
                "sender_name": sender_name,
            }),
            tag: Some(format!("conversation:{}", message.conversation_id)),
        };

        match repository::create(state.db(), user_id, &payload).await {
            Ok(notification_id) => {
                let job = NotificationJob {
                    user_id,
                    notification_id,
                    payload,
                };
                if state.notification_sender().send(job).await.is_err() {
                    tracing::warn!(user_id = %user_id, "notification worker unavailable");
                }
            }
            Err(error) => {
                tracing::warn!(error = %error, user_id = %user_id, "notification audit creation failed");
            }
        }
    }
}
