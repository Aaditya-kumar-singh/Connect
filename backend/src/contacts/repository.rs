use crate::{contacts::models::*, domain::errors::AppError};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list_contacts(pool: &PgPool, user_id: Uuid) -> Result<Vec<Contact>, AppError> {
    sqlx::query_as::<_, Contact>(
        "SELECT c.user_id,c.contact_user_id,c.nickname,c.created_at,u.display_name,p.avatar_url
         FROM contacts c JOIN users u ON u.id=c.contact_user_id
         LEFT JOIN user_profiles p ON p.user_id=u.id
         WHERE c.user_id=$1 ORDER BY u.display_name",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn add_contact(
    pool: &PgPool,
    user_id: Uuid,
    contact_user_id: Uuid,
    nickname: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query("INSERT INTO contacts (user_id,contact_user_id,nickname) VALUES ($1,$2,$3) ON CONFLICT DO NOTHING")
        .bind(user_id).bind(contact_user_id).bind(nickname).execute(pool).await.map(|_| ()).map_err(AppError::Database)
}

pub async fn remove_contact(
    pool: &PgPool,
    user_id: Uuid,
    contact_user_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM contacts WHERE user_id=$1 AND contact_user_id=$2")
        .bind(user_id)
        .bind(contact_user_id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(AppError::Database)
}

pub async fn list_blocks(pool: &PgPool, user_id: Uuid) -> Result<Vec<Uuid>, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT blocked_id FROM blocked_users WHERE blocker_id=$1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn is_blocked(
    pool: &PgPool,
    blocker_id: Uuid,
    blocked_id: Uuid,
) -> Result<bool, AppError> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM blocked_users WHERE blocker_id=$1 AND blocked_id=$2)",
    )
    .bind(blocker_id)
    .bind(blocked_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn add_block(pool: &PgPool, blocker_id: Uuid, blocked_id: Uuid) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO blocked_users (blocker_id,blocked_id) VALUES ($1,$2) ON CONFLICT DO NOTHING",
    )
    .bind(blocker_id)
    .bind(blocked_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM contacts WHERE user_id=$1 AND contact_user_id=$2")
        .bind(blocker_id)
        .bind(blocked_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn remove_block(
    pool: &PgPool,
    blocker_id: Uuid,
    blocked_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM blocked_users WHERE blocker_id=$1 AND blocked_id=$2")
        .bind(blocker_id)
        .bind(blocked_id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(AppError::Database)
}

pub async fn user_exists(pool: &PgPool, user_id: Uuid) -> Result<bool, AppError> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id=$1 AND status='ACTIVE')",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}
