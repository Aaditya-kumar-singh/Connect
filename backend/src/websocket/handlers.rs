use crate::{
    app_state::AppState,
    auth::service,
    websocket::{connection, manager::ConnectionMetadata},
};
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    http::{HeaderMap, Uri},
    response::Response,
};
use uuid::Uuid;

pub async fn connect(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: Uri,
) -> Response {
    let token = bearer_token(&headers).or_else(|| query_token(&uri));
    let device_id = headers
        .get("x-device-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok());

    // Query-string tokens are retained for legacy WebSocket clients, but
    // Authorization headers are preferred because URLs can leak through logs/history.
    ws.on_upgrade(move |socket| run_upgrade(socket, state, token, device_id))
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(ToOwned::to_owned)
}

fn query_token(uri: &Uri) -> Option<String> {
    uri.query().and_then(|query| {
        query
            .split('&')
            .find_map(|part| part.strip_prefix("token="))
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    })
}

async fn run_upgrade(
    mut socket: WebSocket,
    state: AppState,
    token: Option<String>,
    device_id: Option<Uuid>,
) {
    let Some(token) = token else {
        connection::close_with_code(&mut socket, 4001, "authentication required").await;
        return;
    };
    let Some(device_id) = device_id else {
        connection::close_with_code(&mut socket, 4008, "device id required").await;
        return;
    };

    let auth = match service::authenticate(&state, &token).await {
        Ok(auth) => auth,
        Err(_) => {
            connection::close_with_code(&mut socket, 4001, "invalid access token").await;
            return;
        }
    };

    if auth.device_id != device_id {
        connection::close_with_code(&mut socket, 4001, "device mismatch").await;
        return;
    }

    let metadata = ConnectionMetadata {
        connection_id: Uuid::new_v4(),
        user_id: auth.user_id,
        session_id: auth.session_id,
        device_id: auth.device_id,
    };

    connection::run(socket, state, metadata).await;
}
