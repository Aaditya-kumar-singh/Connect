use crate::{
    app_state::AppState,
    auth::models::AuthenticatedSession,
    domain::errors::AppError,
    media::{models::*, repository, validation},
};
use aws_sdk_s3::presigning::PresigningConfig;
use chrono::Utc;
use image::{DynamicImage, ImageFormat};
use sha2::{Digest, Sha256};
use std::{io::Cursor, time::Duration};
use uuid::Uuid;

pub async fn upload(
    state: &AppState,
    auth: &AuthenticatedSession,
    conversation_id: Uuid,
    filename: &str,
    mime_type: &str,
    bytes: Vec<u8>,
) -> Result<(UploadResponse, crate::messaging::models::Message), AppError> {
    require_member(state, auth.user_id, conversation_id).await?;
    let filename = validation::sanitize_filename(filename);
    let extension = validation::extension(&filename)?;
    let spec = validation::spec_for(mime_type, extension)?;
    if bytes.is_empty() || bytes.len() > spec.max_size {
        return Err(AppError::Validation(
            "File size exceeds the media limit".into(),
        ));
    }
    validation::validate_magic(mime_type, &bytes[..bytes.len().min(16)])?;

    let media_id = Uuid::new_v4();
    let r2_key = format!("{conversation_id}/{media_id}/original.{extension}");
    let (stored_bytes, width, height, thumbnail) = if spec.media_type == "image" {
        image_assets(mime_type, &bytes)
            .map_err(|_| AppError::Validation("Invalid image data".into()))?
    } else {
        (bytes.clone(), None, None, None)
    };

    let checksum = Sha256::digest(&stored_bytes);
    let checksum_sha256 = format!("{checksum:x}");
    let original_key = r2_key.clone();

    state
        .storage()
        .ok_or_else(|| AppError::ServiceUnavailable("Media storage is disabled".into()))?
        .put_object()
        .bucket(&state.config().r2_bucket_name)
        .key(&original_key)
        .body(stored_bytes.clone().into())
        .content_type(mime_type)
        .send()
        .await
        .map_err(|_| AppError::ServiceUnavailable("Media storage unavailable".into()))?;

    let thumbnail_r2_key = if let Some(thumbnail) = thumbnail {
        let key = format!("{conversation_id}/{media_id}/thumbnail.webp");
        state
            .storage()
            .ok_or_else(|| AppError::ServiceUnavailable("Media storage is disabled".into()))?
            .put_object()
            .bucket(&state.config().r2_bucket_name)
            .key(&key)
            .body(thumbnail.into())
            .content_type("image/webp")
            .send()
            .await
            .map_err(|_| AppError::ServiceUnavailable("Media storage unavailable".into()))?;
        Some(key)
    } else {
        None
    };

    let now = Utc::now();
    let media = MediaObject {
        id: media_id,
        uploader_id: auth.user_id,
        file_name: filename,
        file_size: stored_bytes.len() as i64,
        mime_type: mime_type.into(),
        media_type: spec.media_type.into(),
        r2_key: original_key,
        thumbnail_r2_key: thumbnail_r2_key.clone(),
        width,
        height,
        duration_seconds: None,
        checksum_sha256,
        created_at: now,
    };

    let mut tx = state.db().begin().await?;
    let message_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO messages
         (id,client_message_id,conversation_id,sender_id,content,content_type,created_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7)",
    )
    .bind(message_id)
    .bind(Uuid::new_v4())
    .bind(conversation_id)
    .bind(auth.user_id)
    .bind("")
    .bind(spec.media_type)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    repository::insert_media(&mut tx, &media).await?;
    repository::insert_attachment(&mut tx, message_id, media_id).await?;
    sqlx::query("UPDATE conversations SET updated_at=$2 WHERE id=$1")
        .bind(conversation_id)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    let thumbnail_url = match thumbnail_r2_key {
        Some(_) => Some(signed_url(state, &media, true).await?),
        None => None,
    };

    let message = crate::messaging::models::Message {
        id: message_id,
        client_message_id: Uuid::nil(),
        conversation_id,
        sender_id: Some(auth.user_id),
        content: Some(String::new()),
        content_type: spec.media_type.into(),
        reply_to_message_id: None,
        forwarded_from_message_id: None,
        edited_at: None,
        deleted_at: None,
        created_at: now,
    };
    Ok((
        UploadResponse {
            media_id,
            message_id,
            media_type: media.media_type,
            file_name: media.file_name,
            file_size: media.file_size,
            mime_type: media.mime_type,
            thumbnail_url,
        },
        message,
    ))
}

pub async fn download_url(
    state: &AppState,
    auth: &AuthenticatedSession,
    media_id: Uuid,
) -> Result<SignedMediaUrl, AppError> {
    let media = repository::find_for_member(state.db(), auth.user_id, media_id).await?;
    Ok(SignedMediaUrl {
        url: signed_url(state, &media, false).await?,
        expires_in: 3600,
    })
}

async fn signed_url(
    state: &AppState,
    media: &MediaObject,
    thumbnail: bool,
) -> Result<String, AppError> {
    let key = if thumbnail {
        media
            .thumbnail_r2_key
            .as_deref()
            .ok_or_else(|| AppError::NotFound("Thumbnail not found".into()))?
    } else {
        &media.r2_key
    };

    let config = PresigningConfig::expires_in(Duration::from_secs(3600))
        .map_err(|_| AppError::Internal("Invalid media URL expiry".into()))?;
    state
        .storage()
        .ok_or_else(|| AppError::ServiceUnavailable("Media storage is disabled".into()))?
        .get_object()
        .bucket(&state.config().r2_bucket_name)
        .key(key)
        .presigned(config)
        .await
        .map(|request| request.uri().to_string())
        .map_err(|_| AppError::ServiceUnavailable("Media storage unavailable".into()))
}

async fn require_member(
    state: &AppState,
    user_id: Uuid,
    conversation_id: Uuid,
) -> Result<(), AppError> {
    let member = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM conversation_members
         WHERE conversation_id=$1 AND user_id=$2 AND left_at IS NULL)",
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_one(state.db())
    .await?;
    if !member {
        return Err(AppError::Forbidden(
            "You are not a member of this conversation".into(),
        ));
    }
    Ok(())
}

type ImageAssets = (Vec<u8>, Option<i32>, Option<i32>, Option<Vec<u8>>);

fn image_assets(mime: &str, bytes: &[u8]) -> Result<ImageAssets, image::ImageError> {
    let image = image::load_from_memory(bytes)?;
    let (width, height) = (image.width() as i32, image.height() as i32);
    let thumbnail = image.thumbnail(300, 300);
    let mut thumb = Cursor::new(Vec::new());
    thumbnail.write_to(&mut thumb, ImageFormat::WebP)?;

    let mut original = Cursor::new(Vec::new());
    match mime {
        "image/jpeg" => {
            DynamicImage::ImageRgb8(image.to_rgb8()).write_to(&mut original, ImageFormat::Jpeg)?
        }
        "image/png" => image.write_to(&mut original, ImageFormat::Png)?,
        "image/gif" => image.write_to(&mut original, ImageFormat::Gif)?,
        "image/webp" => image.write_to(&mut original, ImageFormat::WebP)?,
        "image/heic" => original.get_mut().extend_from_slice(bytes),
        _ => unreachable!(),
    }
    Ok((
        original.into_inner(),
        Some(width),
        Some(height),
        Some(thumb.into_inner()),
    ))
}
