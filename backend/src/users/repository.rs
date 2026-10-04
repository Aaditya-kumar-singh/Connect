use crate::{domain::errors::AppError, users::models::*};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_me(pool: &PgPool, user_id: Uuid) -> Result<MeProfile, AppError> {
    sqlx::query_as::<_, MeProfile>(
        "SELECT u.id,u.email,u.display_name,u.status,p.avatar_url,p.bio,p.phone_number
         FROM users u LEFT JOIN user_profiles p ON p.user_id=u.id WHERE u.id=$1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn ensure_profile(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query("INSERT INTO user_profiles (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING")
        .bind(user_id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(AppError::Database)
}

pub async fn update_me(
    pool: &PgPool,
    user_id: Uuid,
    input: &UpdateProfileRequest,
) -> Result<MeProfile, AppError> {
    ensure_profile(pool, user_id).await?;
    let mut tx = pool.begin().await?;
    if let Some(value) = &input.display_name {
        sqlx::query("UPDATE users SET display_name=$2,updated_at=NOW() WHERE id=$1")
            .bind(user_id)
            .bind(value.trim())
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query(
        "UPDATE user_profiles SET
         bio=COALESCE($2,bio), phone_number=COALESCE($3,phone_number), updated_at=NOW()
         WHERE user_id=$1",
    )
    .bind(user_id)
    .bind(&input.bio)
    .bind(&input.phone_number)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    get_me(pool, user_id).await
}

pub async fn get_public(pool: &PgPool, user_id: Uuid) -> Result<PublicProfile, AppError> {
    sqlx::query_as::<_, PublicProfile>(
        "SELECT u.id,u.display_name,p.avatar_url,p.bio,u.last_seen_at
         FROM users u LEFT JOIN user_profiles p ON p.user_id=u.id
         WHERE u.id=$1 AND u.status='ACTIVE'",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("User not found".into()))
}

pub async fn search(
    pool: &PgPool,
    query: &str,
    limit: i64,
) -> Result<Vec<UserSearchResult>, AppError> {
    let pattern = format!("%{}%", query.trim());
    sqlx::query_as::<_, UserSearchResult>(
        "SELECT u.id,u.display_name,p.avatar_url FROM users u
         LEFT JOIN user_profiles p ON p.user_id=u.id
         WHERE u.status='ACTIVE' AND (u.display_name ILIKE $1 OR u.email ILIKE $1)
         ORDER BY u.display_name ASC LIMIT $2",
    )
    .bind(pattern)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn list_devices(pool: &PgPool, user_id: Uuid) -> Result<Vec<Device>, AppError> {
    sqlx::query_as::<_, Device>(
        "SELECT id,device_name,device_type,push_token,push_provider,last_active_at,created_at
         FROM devices WHERE user_id=$1 ORDER BY last_active_at DESC NULLS LAST,created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn update_device(
    pool: &PgPool,
    user_id: Uuid,
    device_id: Uuid,
    input: &UpdateDeviceRequest,
) -> Result<Device, AppError> {
    sqlx::query_as::<_, Device>(
        "UPDATE devices SET push_token=COALESCE($3,push_token),push_provider=COALESCE($4,push_provider),last_active_at=NOW()
         WHERE id=$1 AND user_id=$2
         RETURNING id,device_name,device_type,push_token,push_provider,last_active_at,created_at",
    ).bind(device_id).bind(user_id).bind(&input.push_token).bind(&input.push_provider)
      .fetch_optional(pool).await.map_err(AppError::Database)?
      .ok_or_else(|| AppError::NotFound("Device not found".into()))
}

pub async fn device_session_ids(
    pool: &PgPool,
    user_id: Uuid,
    device_id: Uuid,
) -> Result<Vec<Uuid>, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM sessions WHERE user_id=$1 AND device_id=$2 AND ended_at IS NULL",
    )
    .bind(user_id)
    .bind(device_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete_device(
    pool: &PgPool,
    user_id: Uuid,
    device_id: Uuid,
) -> Result<Vec<Uuid>, AppError> {
    let ids = device_session_ids(pool, user_id, device_id).await?;
    let result = sqlx::query("DELETE FROM devices WHERE id=$1 AND user_id=$2")
        .bind(device_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Device not found".into()));
    }
    Ok(ids)
}

pub async fn list_sessions(pool: &PgPool, user_id: Uuid) -> Result<Vec<Session>, AppError> {
    sqlx::query_as::<_, Session>(
        "SELECT s.id,s.device_id,d.device_name,d.device_type,s.created_at,s.expires_at
         FROM sessions s JOIN devices d ON d.id=s.device_id
         WHERE s.user_id=$1 AND s.ended_at IS NULL AND s.expires_at>NOW()
         ORDER BY s.created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}
