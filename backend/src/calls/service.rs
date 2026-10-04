use crate::{
    app_state::AppState,
    auth::models::AuthenticatedSession,
    calls::{models::*, repository},
    contacts::repository as contact_repo,
    domain::errors::AppError,
    messaging::handlers::deliver_to_user,
    notifications::{
        models::{NotificationJob, NotificationPayload},
        repository as notification_repo,
    },
};
use base64::Engine;
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde_json::json;
use sha1::Sha1;
use uuid::Uuid;

type HmacSha1 = Hmac<Sha1>;

pub async fn offer(
    state: &AppState,
    auth: &AuthenticatedSession,
    input: OfferInput,
    request_id: Uuid,
) -> Result<(String, Uuid, Uuid), AppError> {
    if input.callee_id == auth.user_id {
        return Err(AppError::Validation("Cannot call yourself".into()));
    }
    let call_type = input.call_type.to_ascii_uppercase();
    if !matches!(call_type.as_str(), "AUDIO" | "VIDEO") {
        return Err(AppError::Validation(
            "call_type must be audio or video".into(),
        ));
    }
    if input.sdp_offer.is_empty() || input.sdp_offer.len() > 64 * 1024 {
        return Err(AppError::Validation("Invalid SDP offer".into()));
    }
    if contact_repo::is_blocked(state.db(), auth.user_id, input.callee_id).await?
        || contact_repo::is_blocked(state.db(), input.callee_id, auth.user_id).await?
    {
        return Err(AppError::Forbidden("User is blocked".into()));
    }
    if !contact_repo::user_exists(state.db(), input.callee_id).await? {
        return Err(AppError::NotFound("User not found".into()));
    }

    let conversation_id =
        repository::conversation_for_call(state.db(), auth.user_id, input.callee_id).await?;
    let call = repository::create(
        state.db(),
        auth.user_id,
        input.callee_id,
        conversation_id,
        &call_type,
    )
    .await?;

    let caller_name = repository::caller_display_name(state.db(), auth.user_id).await?;
    let payload = ServerCallOffer {
        call_id: call.id,
        caller_id: auth.user_id,
        caller_display_name: caller_name.clone(),
        call_type: call_type.to_ascii_lowercase(),
        sdp_offer: input.sdp_offer,
    };
    repository::set_ringing(state.db(), call.id).await?;
    let offer_frame = frame("call.offer", request_id, &payload);
    deliver_to_user(state, input.callee_id, offer_frame).await?;

    let ringing = frame(
        "call.ringing",
        request_id,
        &CallIdPayload { call_id: call.id },
    );
    deliver_to_user(state, auth.user_id, ringing.clone()).await?;

    enqueue_call_notification(state, input.callee_id, &call, auth.user_id, &caller_name).await;

    let timeout_state = state.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        match repository::mark_missed(timeout_state.db(), call.id).await {
            Ok(Some((call, callee_id))) => {
                let participants = repository::participants(timeout_state.db(), call.id)
                    .await
                    .unwrap_or_default();
                let timeout_frame = frame(
                    "call.timeout",
                    Uuid::new_v4(),
                    &CallIdPayload { call_id: call.id },
                );
                for user_id in participants {
                    let _ = deliver_to_user(&timeout_state, user_id, timeout_frame.clone()).await;
                }
                let _ = enqueue_missed_notification(&timeout_state, callee_id, &call).await;
            }
            Ok(None) => {}
            Err(error) => {
                tracing::warn!(%error, call_id=%call.id, "call timeout processing failed")
            }
        }
    });

    Ok((ringing, call.id, input.callee_id))
}

pub async fn answer(
    state: &AppState,
    auth: &AuthenticatedSession,
    input: AnswerInput,
    request_id: Uuid,
) -> Result<String, AppError> {
    if input.sdp_answer.is_empty() || input.sdp_answer.len() > 64 * 1024 {
        return Err(AppError::Validation("Invalid SDP answer".into()));
    }
    let call = repository::answer(state.db(), input.call_id, auth.user_id).await?;
    let target = repository::relay_allowed(state.db(), call.id, auth.user_id).await?;
    let answer_frame = frame(
        "call.answer",
        request_id,
        &ServerAnswer {
            call_id: call.id,
            sdp_answer: input.sdp_answer,
        },
    );
    deliver_to_user(state, target, answer_frame).await?;
    Ok(frame(
        "call.answer.ack",
        request_id,
        &CallIdPayload { call_id: call.id },
    ))
}

pub async fn connected(
    state: &AppState,
    auth: &AuthenticatedSession,
    call_id: Uuid,
    request_id: Uuid,
) -> Result<String, AppError> {
    let target = repository::connected(state.db(), call_id, auth.user_id).await?;
    let frame = frame("call.connected", request_id, &CallIdPayload { call_id });
    deliver_to_user(state, target, frame.clone()).await?;
    Ok(frame)
}

pub async fn ice_candidate(
    state: &AppState,
    auth: &AuthenticatedSession,
    input: IceCandidateInput,
    request_id: Uuid,
) -> Result<String, AppError> {
    if input.candidate.candidate.is_empty() || input.candidate.candidate.len() > 4096 {
        return Err(AppError::Validation("Invalid ICE candidate".into()));
    }
    let target = repository::relay_allowed(state.db(), input.call_id, auth.user_id).await?;
    let candidate_frame = frame("call.ice_candidate", request_id, &input);
    deliver_to_user(state, target, candidate_frame).await?;
    Ok(frame(
        "call.ice_candidate.ack",
        request_id,
        &CallIdPayload {
            call_id: input.call_id,
        },
    ))
}

pub async fn reject(
    state: &AppState,
    auth: &AuthenticatedSession,
    input: RejectInput,
    request_id: Uuid,
) -> Result<String, AppError> {
    let call = repository::reject(state.db(), input.call_id, auth.user_id).await?;
    let target = repository::relay_allowed_for_terminal(state.db(), call.id, auth.user_id).await?;
    let rejected_frame = frame(
        "call.rejected",
        request_id,
        &CallIdPayload { call_id: call.id },
    );
    deliver_to_user(state, target, rejected_frame).await?;
    Ok(frame(
        "call.reject.ack",
        request_id,
        &CallIdPayload { call_id: call.id },
    ))
}

pub async fn end(
    state: &AppState,
    auth: &AuthenticatedSession,
    input: EndInput,
    request_id: Uuid,
) -> Result<String, AppError> {
    let call = repository::end(state.db(), input.call_id, auth.user_id, &input.reason).await?;
    let participants = repository::participants(state.db(), call.id).await?;
    let ended_frame = frame(
        "call.ended",
        request_id,
        &CallEndedPayload {
            call_id: call.id,
            reason: call
                .end_reason
                .clone()
                .unwrap_or_else(|| input.reason.clone()),
            duration_seconds: call.duration_seconds.unwrap_or(0),
        },
    );
    for user_id in participants {
        let _ = deliver_to_user(state, user_id, ended_frame.clone()).await;
    }
    Ok(frame(
        "call.end.ack",
        request_id,
        &CallIdPayload { call_id: call.id },
    ))
}

pub async fn history(
    state: &AppState,
    auth: &AuthenticatedSession,
    limit: Option<u8>,
) -> Result<CallHistoryResponse, AppError> {
    let limit = limit.unwrap_or(50).clamp(1, 100) as i64;
    Ok(CallHistoryResponse {
        items: repository::history(state.db(), auth.user_id, limit).await?,
    })
}

pub async fn ice_config(
    state: &AppState,
    auth: &AuthenticatedSession,
) -> Result<IceConfigResponse, AppError> {
    let mut servers = Vec::new();
    if let Some(url) = state.config().turn_server_url.as_deref() {
        servers.push(IceServer {
            urls: vec![url.to_owned()],
            username: None,
            credential: None,
        });
    }
    let Some(secret) = state.config().turn_secret.as_deref() else {
        return Ok(IceConfigResponse {
            ice_servers: servers,
        });
    };
    let timestamp = Utc::now().timestamp() + 86_400;
    let username = format!("{}:{}", timestamp, auth.user_id);
    let mut mac = HmacSha1::new_from_slice(secret.as_bytes())
        .map_err(|_| AppError::ServiceUnavailable("TURN credential generation failed".into()))?;
    mac.update(username.as_bytes());
    let credential = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    if let Some(url) = state.config().turn_server_url.as_deref() {
        servers.push(IceServer {
            urls: vec![url.to_owned()],
            username: Some(username),
            credential: Some(credential),
        });
    }
    Ok(IceConfigResponse {
        ice_servers: servers,
    })
}

async fn enqueue_call_notification(
    state: &AppState,
    user_id: Uuid,
    call: &CallRecord,
    caller_id: Uuid,
    caller_name: &str,
) {
    let payload = NotificationPayload {
        notification_type: "INCOMING_CALL".into(),
        title: format!("Incoming {} Call", call.call_type.to_ascii_lowercase()),
        body: Some(format!("{caller_name} is calling you")),
        data: json!({
            "type": "INCOMING_CALL",
            "call_id": call.id,
            "caller_id": caller_id,
            "caller_name": caller_name,
            "call_type": call.call_type.to_ascii_lowercase()
        }),
        tag: Some(format!("call:{}", call.id)),
    };
    if let Ok(notification_id) = notification_repo::create(state.db(), user_id, &payload).await {
        let _ = state
            .notification_sender()
            .send(NotificationJob {
                user_id,
                notification_id,
                payload,
            })
            .await;
    }
}

async fn enqueue_missed_notification(
    state: &AppState,
    user_id: Uuid,
    call: &CallRecord,
) -> Result<(), AppError> {
    let caller_name = repository::caller_display_name(state.db(), call.initiated_by).await?;
    let payload = NotificationPayload {
        notification_type: "MISSED_CALL".into(),
        title: "Missed call".into(),
        body: Some(format!("{} called you", caller_name)),
        data: json!({
            "type": "MISSED_CALL",
            "call_id": call.id,
            "caller_id": call.initiated_by,
            "caller_name": caller_name,
            "call_type": call.call_type.to_ascii_lowercase()
        }),
        tag: Some(format!("call:{}", call.id)),
    };
    let notification_id = notification_repo::create(state.db(), user_id, &payload).await?;
    state
        .notification_sender()
        .send(NotificationJob {
            user_id,
            notification_id,
            payload,
        })
        .await
        .map_err(|_| AppError::ServiceUnavailable("Notification worker unavailable".into()))
}

fn frame<T: serde::Serialize>(event_type: &'static str, request_id: Uuid, payload: &T) -> String {
    serde_json::to_string(&crate::websocket::protocol::ServerFrame {
        event_type,
        request_id,
        timestamp: Utc::now(),
        payload,
    })
    .expect("call frame is serializable")
}

#[derive(serde::Serialize)]
struct CallIdPayload {
    call_id: Uuid,
}

#[derive(serde::Serialize)]
struct ServerCallOffer {
    call_id: Uuid,
    caller_id: Uuid,
    caller_display_name: String,
    call_type: String,
    sdp_offer: String,
}

#[derive(serde::Serialize)]
struct ServerAnswer {
    call_id: Uuid,
    sdp_answer: String,
}

#[derive(serde::Serialize)]
struct CallEndedPayload {
    call_id: Uuid,
    reason: String,
    duration_seconds: i32,
}
