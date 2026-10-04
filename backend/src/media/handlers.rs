use crate::{
    app_state::AppState, auth::models::AuthenticatedSession, domain::errors::AppError,
    media::service,
};
use axum::{
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use uuid::Uuid;

pub async fn upload(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<crate::media::models::UploadResponse>), AppError> {
    if let Some(value) = headers.get(axum::http::header::CONTENT_LENGTH) {
        if let Ok(value) = value.to_str().unwrap_or("").parse::<usize>() {
            if value > 50 * 1024 * 1024 + 1024 * 1024 {
                return Err(AppError::Validation("Request body is too large".into()));
            }
        }
    }

    let mut conversation_id = None;
    let mut file = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::Validation("Invalid multipart body".into()))?
    {
        let name = field.name().unwrap_or("");
        if name == "conversation_id" {
            conversation_id = Some(
                field
                    .text()
                    .await
                    .map_err(|_| AppError::Validation("Invalid conversation_id".into()))?,
            );
        } else if name == "file" {
            let filename = field.file_name().unwrap_or("file").to_string();
            let mime = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            let bytes = field
                .bytes()
                .await
                .map_err(|_| AppError::Validation("Unable to read upload".into()))?
                .to_vec();
            file = Some((filename, mime, bytes));
        }
    }

    let conversation_id = conversation_id
        .ok_or_else(|| AppError::Validation("conversation_id is required".into()))?
        .parse::<Uuid>()
        .map_err(|_| AppError::Validation("Invalid conversation_id".into()))?;
    let (filename, mime, bytes) =
        file.ok_or_else(|| AppError::Validation("file is required".into()))?;
    let (result, message) =
        service::upload(&state, &auth, conversation_id, &filename, &mime, bytes).await?;
    let frame = serde_json::to_string(&crate::websocket::protocol::ServerFrame {
        event_type: "message.new",
        request_id: Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        payload: crate::messaging::models::MessageNewPayload {
            message: message.clone(),
            attachments: vec![crate::messaging::models::MessageAttachmentPayload {
                media_id: result.media_id,
                file_name: result.file_name.clone(),
                file_size: result.file_size,
                mime_type: result.mime_type.clone(),
                media_type: result.media_type.clone(),
                thumbnail_url: result.thumbnail_url.clone(),
            }],
        },
    })
    .expect("media message frame is serializable");
    let _ = crate::messaging::handlers::deliver_to_conversation(
        &state,
        conversation_id,
        frame,
        Uuid::nil(),
    )
    .await;
    crate::notifications::service::enqueue_new_message(&state, &message).await;
    Ok((StatusCode::CREATED, Json(result)))
}

pub async fn download_url(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Path(media_id): Path<Uuid>,
) -> Result<Json<crate::media::models::SignedMediaUrl>, AppError> {
    Ok(Json(service::download_url(&state, &auth, media_id).await?))
}
