use crate::{domain::errors::AppError, media::models::MediaObject};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

pub async fn insert_media(
    tx: &mut Transaction<'_, Postgres>,
    media: &MediaObject,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO media_objects
         (id,uploader_id,file_name,file_size,mime_type,media_type,r2_key,thumbnail_r2_key,
          width,height,duration_seconds,checksum_sha256,created_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)",
    )
    .bind(media.id)
    .bind(media.uploader_id)
    .bind(&media.file_name)
    .bind(media.file_size)
    .bind(&media.mime_type)
    .bind(&media.media_type)
    .bind(&media.r2_key)
    .bind(&media.thumbnail_r2_key)
    .bind(media.width)
    .bind(media.height)
    .bind(media.duration_seconds)
    .bind(&media.checksum_sha256)
    .bind(media.created_at)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn insert_attachment(
    tx: &mut Transaction<'_, Postgres>,
    message_id: Uuid,
    media_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query("INSERT INTO message_attachments (message_id,media_id,position) VALUES ($1,$2,0)")
        .bind(message_id)
        .bind(media_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub async fn find_for_member(
    pool: &PgPool,
    user_id: Uuid,
    media_id: Uuid,
) -> Result<MediaObject, AppError> {
    sqlx::query_as::<_, MediaObject>(
        "SELECT mo.id,mo.uploader_id,mo.file_name,mo.file_size,mo.mime_type,mo.media_type,
                mo.r2_key,mo.thumbnail_r2_key,mo.width,mo.height,mo.duration_seconds,
                mo.checksum_sha256,mo.created_at
         FROM media_objects mo
         JOIN message_attachments ma ON ma.media_id=mo.id
         JOIN messages m ON m.id=ma.message_id
         JOIN conversation_members cm
           ON cm.conversation_id=m.conversation_id
          AND cm.user_id=$1
          AND cm.left_at IS NULL
         WHERE mo.id=$2 AND m.deleted_at IS NULL
         LIMIT 1",
    )
    .bind(user_id)
    .bind(media_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Media not found".into()))
}
