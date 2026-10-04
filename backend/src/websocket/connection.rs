use crate::{
    app_state::AppState,
    auth::models::AuthenticatedSession,
    domain::errors::AppError,
    messaging::handlers as message_handlers,
    websocket::{
        manager::{ConnectionHandle, ConnectionMetadata},
        protocol::{self, ClientFrame},
        HEARTBEAT_TIMEOUT_SECONDS, REGISTRY_TTL_SECONDS,
    },
};
use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::sync::mpsc;
use uuid::Uuid;

pub async fn run(mut socket: WebSocket, state: AppState, metadata: ConnectionMetadata) {
    let connection_id = metadata.connection_id;
    let registry_member = format!(
        "{}:{}:{}",
        state.instance_id(),
        connection_id,
        metadata.device_id
    );
    let (sender, mut receiver) = mpsc::channel::<Message>(32);

    if !state
        .connection_manager()
        .insert(ConnectionHandle {
            metadata: metadata.clone(),
            sender,
        })
        .await
    {
        close_with_code(&mut socket, 1011, "connection registration failed").await;
        return;
    }
    state.metrics().ws_connected();

    if let Err(error) = register_in_redis(&state, &metadata, &registry_member).await {
        tracing::warn!(%error, connection_id = %connection_id, "websocket Redis registry unavailable");
    }
    if let Err(error) = crate::presence::connect(&state, metadata.user_id).await {
        tracing::warn!(%error, user_id = %metadata.user_id, "presence online update unavailable");
    }

    let connected = Message::Text(protocol::connected_frame(
        metadata.session_id,
        metadata.user_id,
        metadata.device_id,
    ));
    state.metrics().ws_message_sent("connected");
    if socket.send(connected).await.is_err() {
        cleanup(&state, &metadata, &registry_member).await;
        return;
    }

    let (mut sink, mut stream) = socket.split();
    let mut heartbeat = tokio::time::interval(Duration::from_secs(15));
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut last_activity = tokio::time::Instant::now();
    let auth = AuthenticatedSession {
        user_id: metadata.user_id,
        session_id: metadata.session_id,
        device_id: metadata.device_id,
        access_token_hash: String::new(),
    };

    loop {
        tokio::select! {
            Some(outbound) = receiver.recv() => {
                if matches!(&outbound, Message::Text(_)) {
                    state.metrics().ws_message_sent("outbound");
                }
                if sink.send(outbound).await.is_err() {
                    break;
                }
            }
            _ = heartbeat.tick() => {
                if last_activity.elapsed() >= Duration::from_secs(HEARTBEAT_TIMEOUT_SECONDS) {
                    let _ = sink.send(Message::Close(Some(CloseFrame {
                        code: 4000,
                        reason: "heartbeat timeout".to_owned().into(),
                    }))).await;
                    break;
                }
                if refresh_registry(&state, &metadata).await.is_err() {
                    tracing::debug!(connection_id = %connection_id, "websocket Redis registry refresh failed");
                }
                if let Err(error) = crate::presence::heartbeat(&state, metadata.user_id).await {
                    tracing::debug!(%error, user_id = %metadata.user_id, "presence heartbeat refresh failed");
                }
                if sink.send(Message::Ping(Vec::new())).await.is_err() {
                    break;
                }
            }
            incoming = stream.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        last_activity = tokio::time::Instant::now();
                        if !handle_text(&mut sink, &state, &auth, connection_id, &text).await {
                            break;
                        }
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        last_activity = tokio::time::Instant::now();
                        if sink.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        last_activity = tokio::time::Instant::now();
                    }
                    Some(Ok(Message::Close(frame))) => {
                        let _ = sink.send(Message::Close(frame)).await;
                        break;
                    }
                    Some(Ok(Message::Binary(_))) => {
                        let _ = close_with_sink(&mut sink, 1003, "text frames only").await;
                        break;
                    }
                    Some(Err(error)) => {
                        tracing::debug!(%error, connection_id = %connection_id, "websocket receive error");
                        break;
                    }
                    None => break,
                }
            }
        }
    }

    cleanup(&state, &metadata, &registry_member).await;
}

async fn handle_text(
    sink: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    state: &AppState,
    auth: &AuthenticatedSession,
    connection_id: Uuid,
    text: &str,
) -> bool {
    let frame = match serde_json::from_str::<ClientFrame>(text) {
        Ok(frame) => frame,
        Err(_) => {
            let _ = close_with_sink(sink, 4002, "invalid frame").await;
            return false;
        }
    };

    state.metrics().ws_message_received(&frame.event_type);

    let Some(request_id) = frame.request_id else {
        return send_error(
            sink,
            None,
            &AppError::Validation("request_id is required".into()),
        )
        .await;
    };

    if let Some((limit, window)) = match frame.event_type.as_str() {
        "message.send" => Some((30, 60)),
        "message.edit" => Some((10, 60)),
        "message.delete" => Some((10, 60)),
        "message.react" => Some((20, 60)),
        "message.delivered" => Some((60, 60)),
        "message.read" => Some((30, 60)),
        "typing.start" => Some((30, 60)),
        "call.offer" => Some((5, 60)),
        "call.answer" => Some((10, 60)),
        "call.connected" => Some((5, 60)),
        "call.ice_candidate" => Some((120, 60)),
        "call.reject" => Some((10, 60)),
        "call.end" => Some((10, 60)),
        _ => None,
    } {
        let key = format!("rate:ws:{}:{}", auth.user_id, frame.event_type);
        if let Err(error) = crate::auth::rate_limit::check_key(state, &key, limit, window).await {
            return send_error(sink, Some(request_id), &error).await;
        }
        let total_key = format!("rate:ws:{}:total", auth.user_id);
        if let Err(error) = crate::auth::rate_limit::check_key(state, &total_key, 120, 60).await {
            return send_error(sink, Some(request_id), &error).await;
        }
    }

    match frame.event_type.as_str() {
        "auth.ping" => sink
            .send(Message::Text(protocol::pong_frame(Some(request_id))))
            .await
            .is_ok(),
        "message.send" => {
            match message_handlers::handle_send_payload(state, auth, request_id, frame.payload)
                .await
            {
                Ok((ack, broadcast)) => {
                    if sink.send(Message::Text(ack)).await.is_err() {
                        return false;
                    }
                    if let Some((conversation_id, event)) = broadcast {
                        let _ = message_handlers::deliver_to_conversation(
                            state,
                            conversation_id,
                            event,
                            connection_id,
                        )
                        .await;
                    }
                    true
                }
                Err(error) => send_error(sink, Some(request_id), &error).await,
            }
        }
        "message.edit" => match message_handlers::handle_edit(state, auth, frame.payload).await {
            Ok((conversation_id, event)) => {
                let _ = message_handlers::deliver_to_conversation(
                    state,
                    conversation_id,
                    event,
                    connection_id,
                )
                .await;
                true
            }
            Err(error) => send_error(sink, Some(request_id), &error).await,
        },
        "message.delete" => match message_handlers::handle_delete(state, auth, frame.payload).await
        {
            Ok((conversation_id, event)) => {
                let _ = message_handlers::deliver_to_conversation(
                    state,
                    conversation_id,
                    event,
                    connection_id,
                )
                .await;
                true
            }
            Err(error) => send_error(sink, Some(request_id), &error).await,
        },
        "message.react" => {
            match message_handlers::handle_reaction(state, auth, frame.payload).await {
                Ok((conversation_id, Some(event))) => {
                    let _ = message_handlers::deliver_to_conversation(
                        state,
                        conversation_id,
                        event,
                        connection_id,
                    )
                    .await;
                    true
                }
                Ok((_, None)) => true,
                Err(error) => send_error(sink, Some(request_id), &error).await,
            }
        }
        "message.delivered" => {
            match message_handlers::handle_delivered(state, auth, request_id, frame.payload).await {
                Ok(()) => true,
                Err(error) => send_error(sink, Some(request_id), &error).await,
            }
        }
        "message.read" => {
            match message_handlers::handle_read(state, auth, request_id, frame.payload).await {
                Ok(()) => true,
                Err(error) => send_error(sink, Some(request_id), &error).await,
            }
        }
        "typing.start" => {
            let input = match serde_json::from_value::<crate::typing::TypingPayload>(frame.payload)
            {
                Ok(input) => input,
                Err(_) => {
                    return send_error(
                        sink,
                        Some(request_id),
                        &AppError::Validation("Invalid event payload".into()),
                    )
                    .await;
                }
            };
            match crate::typing::start(state, auth, input.conversation_id).await {
                Ok(_) => true,
                Err(error) => send_error(sink, Some(request_id), &error).await,
            }
        }
        "call.offer" => {
            match serde_json::from_value::<crate::calls::models::OfferInput>(frame.payload) {
                Ok(input) => {
                    match crate::calls::service::offer(state, auth, input, request_id).await {
                        Ok((ringing, _, _)) => sink.send(Message::Text(ringing)).await.is_ok(),
                        Err(error) => send_error(sink, Some(request_id), &error).await,
                    }
                }
                Err(_) => {
                    send_error(
                        sink,
                        Some(request_id),
                        &AppError::Validation("Invalid call offer payload".into()),
                    )
                    .await
                }
            }
        }
        "call.answer" => {
            match serde_json::from_value::<crate::calls::models::AnswerInput>(frame.payload) {
                Ok(input) => {
                    match crate::calls::service::answer(state, auth, input, request_id).await {
                        Ok(ack) => sink.send(Message::Text(ack)).await.is_ok(),
                        Err(error) => send_error(sink, Some(request_id), &error).await,
                    }
                }
                Err(_) => {
                    send_error(
                        sink,
                        Some(request_id),
                        &AppError::Validation("Invalid call answer payload".into()),
                    )
                    .await
                }
            }
        }
        "call.connected" => match frame
            .payload
            .get("call_id")
            .and_then(|v| v.as_str())
            .and_then(|v| Uuid::parse_str(v).ok())
        {
            Some(call_id) => {
                match crate::calls::service::connected(state, auth, call_id, request_id).await {
                    Ok(event) => sink.send(Message::Text(event)).await.is_ok(),
                    Err(error) => send_error(sink, Some(request_id), &error).await,
                }
            }
            None => {
                send_error(
                    sink,
                    Some(request_id),
                    &AppError::Validation("Invalid call_id".into()),
                )
                .await
            }
        },
        "call.ice_candidate" => {
            match serde_json::from_value::<crate::calls::models::IceCandidateInput>(frame.payload) {
                Ok(input) => {
                    match crate::calls::service::ice_candidate(state, auth, input, request_id).await
                    {
                        Ok(ack) => sink.send(Message::Text(ack)).await.is_ok(),
                        Err(error) => send_error(sink, Some(request_id), &error).await,
                    }
                }
                Err(_) => {
                    send_error(
                        sink,
                        Some(request_id),
                        &AppError::Validation("Invalid ICE candidate payload".into()),
                    )
                    .await
                }
            }
        }
        "call.reject" => {
            match serde_json::from_value::<crate::calls::models::RejectInput>(frame.payload) {
                Ok(input) => {
                    match crate::calls::service::reject(state, auth, input, request_id).await {
                        Ok(ack) => sink.send(Message::Text(ack)).await.is_ok(),
                        Err(error) => send_error(sink, Some(request_id), &error).await,
                    }
                }
                Err(_) => {
                    send_error(
                        sink,
                        Some(request_id),
                        &AppError::Validation("Invalid call reject payload".into()),
                    )
                    .await
                }
            }
        }
        "call.end" => match serde_json::from_value::<crate::calls::models::EndInput>(frame.payload)
        {
            Ok(input) => match crate::calls::service::end(state, auth, input, request_id).await {
                Ok(ack) => sink.send(Message::Text(ack)).await.is_ok(),
                Err(error) => send_error(sink, Some(request_id), &error).await,
            },
            Err(_) => {
                send_error(
                    sink,
                    Some(request_id),
                    &AppError::Validation("Invalid call end payload".into()),
                )
                .await
            }
        },
        "typing.stop" => {
            let input = match serde_json::from_value::<crate::typing::TypingPayload>(frame.payload)
            {
                Ok(input) => input,
                Err(_) => {
                    return send_error(
                        sink,
                        Some(request_id),
                        &AppError::Validation("Invalid event payload".into()),
                    )
                    .await;
                }
            };
            match crate::typing::stop(state, auth, input.conversation_id).await {
                Ok(_) => true,
                Err(error) => send_error(sink, Some(request_id), &error).await,
            }
        }
        _ => {
            send_error(
                sink,
                Some(request_id),
                &AppError::Validation("Unsupported WebSocket event".into()),
            )
            .await
        }
    }
}

async fn send_error(
    sink: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    request_id: Option<Uuid>,
    error: &AppError,
) -> bool {
    sink.send(Message::Text(message_handlers::error_frame(
        request_id, error,
    )))
    .await
    .is_ok()
}

async fn register_in_redis(
    state: &AppState,
    metadata: &ConnectionMetadata,
    member: &str,
) -> Result<(), redis::RedisError> {
    let mut conn = state.redis().get_multiplexed_async_connection().await?;
    let key = format!("connections:{}", metadata.user_id);
    let _: () = redis::cmd("SADD")
        .arg(&key)
        .arg(member)
        .query_async(&mut conn)
        .await?;
    let _: () = redis::cmd("EXPIRE")
        .arg(&key)
        .arg(REGISTRY_TTL_SECONDS)
        .query_async(&mut conn)
        .await?;
    Ok(())
}

async fn refresh_registry(
    state: &AppState,
    metadata: &ConnectionMetadata,
) -> Result<(), redis::RedisError> {
    let mut conn = state.redis().get_multiplexed_async_connection().await?;
    let key = format!("connections:{}", metadata.user_id);
    let _: () = redis::cmd("EXPIRE")
        .arg(&key)
        .arg(REGISTRY_TTL_SECONDS)
        .query_async(&mut conn)
        .await?;
    Ok(())
}

async fn cleanup(state: &AppState, metadata: &ConnectionMetadata, member: &str) {
    state
        .connection_manager()
        .remove(metadata.connection_id)
        .await;
    state.metrics().ws_disconnected();
    if let Err(error) = crate::presence::disconnect(state, metadata.user_id, member).await {
        tracing::warn!(%error, user_id = %metadata.user_id, "presence offline update unavailable");
    }
    match state.redis().get_multiplexed_async_connection().await {
        Ok(mut conn) => {
            let key = format!("connections:{}", metadata.user_id);
            let _: Result<(), _> = redis::cmd("SREM")
                .arg(&key)
                .arg(member)
                .query_async(&mut conn)
                .await;
            let _: Result<(), _> = redis::cmd("EXPIRE")
                .arg(&key)
                .arg(REGISTRY_TTL_SECONDS)
                .query_async(&mut conn)
                .await;
        }
        Err(error) => tracing::debug!(%error, "websocket Redis cleanup unavailable"),
    }
}

pub async fn close_with_code(socket: &mut WebSocket, code: u16, reason: &str) {
    let _ = socket
        .send(Message::Close(Some(CloseFrame {
            code,
            reason: reason.to_owned().into(),
        })))
        .await;
}

async fn close_with_sink(
    sink: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    code: u16,
    reason: &str,
) -> Result<(), axum::Error> {
    sink.send(Message::Close(Some(CloseFrame {
        code,
        reason: reason.to_owned().into(),
    })))
    .await
    .map_err(axum::Error::new)
}
