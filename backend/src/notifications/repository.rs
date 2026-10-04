use crate::notifications::models::{DevicePushTarget, NotificationPayload};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    payload: &NotificationPayload,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO notifications (id,user_id,notification_type,title,body,data)
         VALUES ($1,$2,$3,$4,$5,$6)",
    )
    .bind(id)
    .bind(user_id)
    .bind(&payload.notification_type)
    .bind(&payload.title)
    .bind(&payload.body)
    .bind(&payload.data)
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn targets(pool: &PgPool, user_id: Uuid) -> Result<Vec<DevicePushTarget>, sqlx::Error> {
    sqlx::query_as::<_, (Uuid, String, String)>(
        "SELECT id,push_token,push_provider
         FROM devices
         WHERE user_id=$1 AND push_token IS NOT NULL AND push_provider IS NOT NULL",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(device_id, token, provider)| DevicePushTarget {
                device_id,
                token,
                provider,
            })
            .collect()
    })
}

pub async fn mark_sent(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE notifications SET sent_at=$2 WHERE id=$1")
        .bind(id)
        .bind(Utc::now())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn remove_token(pool: &PgPool, device_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE devices SET push_token=NULL,push_provider=NULL WHERE id=$1")
        .bind(device_id)
        .execute(pool)
        .await?;
    Ok(())
}
