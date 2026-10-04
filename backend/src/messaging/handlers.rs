use crate::{
    app_state::AppState,
    auth::models::AuthenticatedSession,
    domain::errors::AppError,
    messaging::{
        models::{
            DeleteMessagePayload, EditMessagePayload, ErrorPayload, ForwardMessageRequest,
            MessageAckPayload, MessageDeletedPayload, MessageDeliveredPayload,
            MessageEditedPayload, MessageNewPayload, MessageReactionPayload, MessageReadPayload,
            MessageStatusPayload, ReactionPayload, SendMessagePayload,
        },
        service,
    },
    websocket::{protocol::ServerFrame, pubsub},
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

pub fn error_frame(request_id: Option<Uuid>, error: &AppError) -> String {
    let (code, message) = match error {
        AppError::Forbidden(message) => ("FORBIDDEN", message.clone()),
        AppError::NotFound(message) => ("NOT_FOUND", message.clone()),
        AppError::Conflict(message) => ("CONFLICT", message.clone()),
        AppError::Validation(message) => ("VALIDATION_ERROR", message.clone()),
        AppError::RateLimited => ("RATE_LIMITED", "Too many requests".into()),
        AppError::Gone(message) => ("GONE", message.clone()),
        AppError::ServiceUnavailable(message) => ("SERVICE_UNAVAILABLE", message.clone()),
        _ => ("INTERNAL_ERROR", "Internal server error".into()),
    };
    serde_json::to_string(&ServerFrame {
        event_type: "error",
        request_id: request_id.unwrap_or_else(Uuid::new_v4),
        timestamp: Utc::now(),
        payload: ErrorPayload { code, message },
    })
    .expect("error frame is serializable")
}

pub fn ack_frame(request_id: Uuid, payload: MessageAckPayload) -> String {
    serde_json::to_string(&ServerFrame {
        event_type: "message.ack",
        request_id,
        timestamp: Utc::now(),
        payload,
    })
    .expect("message ack is serializable")
}

fn new_frame(payload: MessageNewPayload) -> String {
    serde_json::to_string(&ServerFrame {
        event_type: "message.new",
        request_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        payload,
    })
    .expect("message new is serializable")
}

fn edited_frame(payload: MessageEditedPayload) -> String {
    serde_json::to_string(&ServerFrame {
        event_type: "message.edited",
        request_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        payload,
    })
    .expect("message edited is serializable")
}

fn deleted_frame(payload: MessageDeletedPayload) -> String {
    serde_json::to_string(&ServerFrame {
        event_type: "message.deleted",
        request_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        payload,
    })
    .expect("message deleted is serializable")
}

fn reaction_frame(payload: MessageReactionPayload) -> String {
    serde_json::to_string(&ServerFrame {
        event_type: "message.reaction",
        request_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        payload,
    })
    .expect("message reaction is serializable")
}

pub fn status_frame(request_id: Uuid, payload: MessageStatusPayload) -> String {
    serde_json::to_string(&ServerFrame {
        event_type: "message.status",
        request_id,
        timestamp: Utc::now(),
        payload,
    })
    .expect("message status is serializable")
}

pub async fn deliver_to_user(
    state: &AppState,
    user_id: Uuid,
    frame: String,
) -> Result<(), AppError> {
    let _ = state
        .connection_manager()
        .send_to_user(
            user_id,
            axum::extract::ws::Message::Text(frame.clone()),
            None,
        )
        .await;
    if let Err(error) = pubsub::publish_user_frame(state, user_id, &frame).await {
        tracing::debug!(%error, %user_id, "cross-instance status delivery publish failed");
    }
    Ok(())
}

async fn conversation_members(
    state: &AppState,
    conversation_id: Uuid,
) -> Result<Vec<Uuid>, AppError> {
    Ok(sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM conversation_members WHERE conversation_id=$1 AND left_at IS NULL",
    )
    .bind(conversation_id)
    .fetch_all(state.db())
    .await?)
}

pub async fn deliver_to_conversation(
    state: &AppState,
    conversation_id: Uuid,
    frame: String,
    current_connection: Uuid,
) -> Result<(), AppError> {
    let members = conversation_members(state, conversation_id).await?;
    for user_id in members {
        let exclude = if state
            .connection_manager()
            .contains(current_connection)
            .await
        {
            Some(current_connection)
        } else {
            None
        };
        let _ = state
            .connection_manager()
            .send_to_user(
                user_id,
                axum::extract::ws::Message::Text(frame.clone()),
                exclude,
            )
            .await;
        if let Err(error) = pubsub::publish_user_frame(state, user_id, &frame).await {
            tracing::debug!(%error, %user_id, "cross-instance message delivery publish failed");
        }
    }
    Ok(())
}

pub async fn history(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Path(conversation_id): Path<Uuid>,
    Query(query): Query<crate::messaging::models::MessageHistoryQuery>,
) -> Result<Json<crate::messaging::models::MessageHistory>, AppError> {
    Ok(Json(
        service::history(
            &state,
            &auth,
            conversation_id,
            query.limit,
            query.cursor.as_deref(),
        )
        .await?,
    ))
}

pub async fn forward(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Json(input): Json<ForwardMessageRequest>,
) -> Result<Json<crate::messaging::models::Message>, AppError> {
    let message = service::forward(&state, &auth, &input).await?;
    let frame = new_frame(MessageNewPayload {
        message: message.clone(),
        attachments: Vec::new(),
    });
    let _ = deliver_to_conversation(&state, message.conversation_id, frame, Uuid::nil()).await;
    Ok(Json(message))
}

pub fn parse_payload<T: serde::de::DeserializeOwned>(payload: Value) -> Result<T, AppError> {
    serde_json::from_value(payload)
        .map_err(|_| AppError::Validation("Invalid event payload".into()))
}

pub async fn handle_send_payload(
    state: &AppState,
    auth: &AuthenticatedSession,
    request_id: Uuid,
    payload: Value,
) -> Result<(String, Option<(Uuid, String)>), AppError> {
    let input: SendMessagePayload = parse_payload(payload)?;
    let (message, created) = service::send(state, auth, &input).await?;
    if created {
        crate::notifications::service::enqueue_new_message(state, &message).await;
    }
    let ack = ack_frame(
        request_id,
        MessageAckPayload {
            client_message_id: message.client_message_id,
            server_message_id: message.id,
            timestamp: message.created_at,
        },
    );
    let broadcast = created.then(|| {
        (
            message.conversation_id,
            new_frame(MessageNewPayload {
                message,
                attachments: Vec::new(),
            }),
        )
    });
    Ok((ack, broadcast))
}

pub async fn handle_edit(
    state: &AppState,
    auth: &AuthenticatedSession,
    payload: Value,
) -> Result<(Uuid, String), AppError> {
    let input: EditMessagePayload = parse_payload(payload)?;
    let message = service::edit(state, auth, input.message_id, &input.content).await?;
    let frame = edited_frame(MessageEditedPayload {
        message_id: message.id,
        content: message.content.unwrap_or_default(),
        edited_at: message.edited_at.unwrap_or_else(Utc::now),
    });
    Ok((message.conversation_id, frame))
}

pub async fn handle_delete(
    state: &AppState,
    auth: &AuthenticatedSession,
    payload: Value,
) -> Result<(Uuid, String), AppError> {
    let input: DeleteMessagePayload = parse_payload(payload)?;
    let message = service::delete(state, auth, input.message_id).await?;
    let frame = deleted_frame(MessageDeletedPayload {
        message_id: message.id,
        deleted_at: message.deleted_at.unwrap_or_else(Utc::now),
    });
    Ok((message.conversation_id, frame))
}

pub async fn handle_delivered(
    state: &AppState,
    auth: &AuthenticatedSession,
    request_id: Uuid,
    payload: Value,
) -> Result<(), AppError> {
    let input: MessageDeliveredPayload = parse_payload(payload)?;
    let statuses = service::delivered(state, auth, &input.message_ids).await?;
    for (message_id, sender_id, timestamp) in statuses {
        let frame = status_frame(
            request_id,
            MessageStatusPayload {
                message_id,
                status: "delivered".into(),
                user_id: auth.user_id,
                timestamp,
            },
        );
        let _ = deliver_to_user(state, sender_id, frame).await;
    }
    Ok(())
}

pub async fn handle_read(
    state: &AppState,
    auth: &AuthenticatedSession,
    request_id: Uuid,
    payload: Value,
) -> Result<(), AppError> {
    let input: MessageReadPayload = parse_payload(payload)?;
    let statuses = service::read(
        state,
        auth,
        input.conversation_id,
        input.last_read_message_id,
    )
    .await?;

    for (message_id, sender_id, timestamp) in &statuses {
        let frame = status_frame(
            request_id,
            MessageStatusPayload {
                message_id: *message_id,
                status: "read".into(),
                user_id: auth.user_id,
                timestamp: *timestamp,
            },
        );
        let _ = deliver_to_user(state, *sender_id, frame).await;
    }

    Ok(())
}

pub async fn handle_reaction(
    state: &AppState,
    auth: &AuthenticatedSession,
    payload: Value,
) -> Result<(Uuid, Option<String>), AppError> {
    let input: ReactionPayload = parse_payload(payload)?;
    let (message_id, conversation_id, user_id, emoji, action, changed) =
        service::react(state, auth, input.message_id, &input.emoji, &input.action).await?;
    let frame = changed.then(|| {
        reaction_frame(MessageReactionPayload {
            message_id,
            user_id,
            emoji,
            action,
        })
    });
    Ok((conversation_id, frame))
}
