use crate::auth::models::{RefreshTokenRecord, User};
use crate::domain::errors::AppError;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

pub async fn create_user(
    tx: &mut Transaction<'_, Postgres>,
    email: &str,
    password_hash: &str,
    display_name: &str,
) -> Result<User, AppError> {
    sqlx::query_as::<_, User>(
        "INSERT INTO users (email, password_hash, display_name) VALUES ($1,$2,$3)
         RETURNING id,email,password_hash,display_name,status",
    )
    .bind(email)
    .bind(password_hash)
    .bind(display_name)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db)
}

pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, AppError> {
    sqlx::query_as::<_, User>(
        "SELECT id,email,password_hash,display_name,status FROM users WHERE email=$1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .map_err(map_db)
}

pub async fn create_email_verification(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    otp_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), AppError> {
    sqlx::query("INSERT INTO email_verifications (user_id,otp_hash,expires_at) VALUES ($1,$2,$3)")
        .bind(user_id)
        .bind(otp_hash)
        .bind(expires_at)
        .execute(&mut **tx)
        .await
        .map(|_| ())
        .map_err(map_db)
}

pub async fn verify_email(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    otp_hash: &str,
) -> Result<(), AppError> {
    let row = sqlx::query_as::<_, (Uuid, i16, DateTime<Utc>, Option<DateTime<Utc>>)>(
        "SELECT id,attempts,expires_at,verified_at FROM email_verifications
         WHERE user_id=$1 AND verified_at IS NULL ORDER BY created_at DESC LIMIT 1 FOR UPDATE",
    )
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db)?
    .ok_or_else(|| AppError::Unauthorized("Invalid OTP".into()))?;
    if row.2 <= Utc::now() {
        return Err(AppError::Gone("OTP expired".into()));
    }
    if row.1 >= 5 {
        return Err(AppError::RateLimited);
    }
    let stored =
        sqlx::query_scalar::<_, String>("SELECT otp_hash FROM email_verifications WHERE id=$1")
            .bind(row.0)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_db)?;
    if stored != otp_hash {
        sqlx::query("UPDATE email_verifications SET attempts=attempts+1 WHERE id=$1")
            .bind(row.0)
            .execute(&mut **tx)
            .await
            .map_err(map_db)?;
        return Err(AppError::Unauthorized("Invalid OTP".into()));
    }
    sqlx::query("UPDATE email_verifications SET verified_at=NOW() WHERE id=$1")
        .bind(row.0)
        .execute(&mut **tx)
        .await
        .map_err(map_db)?;
    sqlx::query("UPDATE users SET status='ACTIVE', updated_at=NOW() WHERE id=$1")
        .bind(user_id)
        .execute(&mut **tx)
        .await
        .map_err(map_db)?;
    Ok(())
}

pub async fn upsert_device(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    device_id: Uuid,
    device_name: Option<&str>,
    device_type: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO devices (id,user_id,device_name,device_type,last_active_at) VALUES ($1,$2,$3,$4,NOW())
         ON CONFLICT (id) DO UPDATE SET device_name=EXCLUDED.device_name, device_type=EXCLUDED.device_type,last_active_at=NOW()
         WHERE devices.user_id=EXCLUDED.user_id"
    ).bind(device_id).bind(user_id).bind(device_name).bind(device_type).execute(&mut **tx).await.map(|_| ()).map_err(map_db)
}

pub async fn create_session(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    device_id: Uuid,
    expires_at: DateTime<Utc>,
) -> Result<Uuid, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO sessions (user_id,device_id,expires_at) VALUES ($1,$2,$3) RETURNING id",
    )
    .bind(user_id)
    .bind(device_id)
    .bind(expires_at)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db)
}

pub async fn session_device_id(pool: &PgPool, session_id: Uuid) -> Result<Uuid, AppError> {
    sqlx::query_scalar::<_, Uuid>("SELECT device_id FROM sessions WHERE id=$1 AND ended_at IS NULL")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .map_err(map_db)
}

pub async fn active_session_ids(pool: &PgPool, user_id: Uuid) -> Result<Vec<Uuid>, AppError> {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM sessions WHERE user_id=$1 AND ended_at IS NULL")
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(map_db)
}

pub async fn create_refresh_token(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    session_id: Uuid,
    hash: &str,
    family_id: Uuid,
    expires_at: DateTime<Utc>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO refresh_tokens (user_id,session_id,token_hash,family_id,expires_at) VALUES ($1,$2,$3,$4,$5)"
    ).bind(user_id).bind(session_id).bind(hash).bind(family_id).bind(expires_at).execute(&mut **tx).await.map(|_| ()).map_err(map_db)
}

pub async fn find_refresh_token(
    pool: &PgPool,
    hash: &str,
) -> Result<Option<RefreshTokenRecord>, AppError> {
    sqlx::query_as::<_, RefreshTokenRecord>(
        "SELECT id,user_id,session_id,token_hash,family_id,revoked_at,expires_at FROM refresh_tokens WHERE token_hash=$1"
    ).bind(hash).fetch_optional(pool).await.map_err(map_db)
}

pub async fn rotate_refresh_token(
    tx: &mut Transaction<'_, Postgres>,
    old_id: Uuid,
    new_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<bool, AppError> {
    let rotated = sqlx::query_as::<_, (Uuid, Uuid, Uuid)>(
        "UPDATE refresh_tokens
         SET revoked_at=NOW()
         WHERE id=$1 AND revoked_at IS NULL
         RETURNING user_id,session_id,family_id",
    )
    .bind(old_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db)?;

    let Some((user_id, session_id, family_id)) = rotated else {
        return Ok(false);
    };

    sqlx::query(
        "INSERT INTO refresh_tokens (user_id,session_id,token_hash,family_id,expires_at)
         VALUES ($1,$2,$3,$4,$5)",
    )
    .bind(user_id)
    .bind(session_id)
    .bind(new_hash)
    .bind(family_id)
    .bind(expires_at)
    .execute(&mut **tx)
    .await
    .map(|_| true)
    .map_err(map_db)
}

pub async fn revoke_refresh_family(
    tx: &mut Transaction<'_, Postgres>,
    family_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE refresh_tokens SET revoked_at=COALESCE(revoked_at,NOW()) WHERE family_id=$1",
    )
    .bind(family_id)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(map_db)
}

pub async fn revoke_session(
    tx: &mut Transaction<'_, Postgres>,
    session_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query("UPDATE sessions SET ended_at=COALESCE(ended_at,NOW()) WHERE id=$1")
        .bind(session_id)
        .execute(&mut **tx)
        .await
        .map(|_| ())
        .map_err(map_db)?;
    sqlx::query(
        "UPDATE refresh_tokens SET revoked_at=COALESCE(revoked_at,NOW()) WHERE session_id=$1",
    )
    .bind(session_id)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(map_db)
}

pub async fn revoke_all_sessions(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query("UPDATE sessions SET ended_at=COALESCE(ended_at,NOW()) WHERE user_id=$1 AND ended_at IS NULL").bind(user_id).execute(&mut **tx).await.map_err(map_db)?;
    sqlx::query("UPDATE refresh_tokens SET revoked_at=COALESCE(revoked_at,NOW()) WHERE user_id=$1")
        .bind(user_id)
        .execute(&mut **tx)
        .await
        .map(|_| ())
        .map_err(map_db)
}

pub async fn create_reset_otp(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    otp_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), AppError> {
    sqlx::query("INSERT INTO otp_requests (user_id,otp_type,otp_hash,expires_at) VALUES ($1,'PASSWORD_RESET',$2,$3)")
        .bind(user_id).bind(otp_hash).bind(expires_at).execute(&mut **tx).await.map(|_| ()).map_err(map_db)
}

pub async fn find_reset_otp(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> Result<Option<(Uuid, String, i16, DateTime<Utc>, Option<DateTime<Utc>>)>, AppError> {
    sqlx::query_as(
        "SELECT id,otp_hash,attempts,expires_at,used_at FROM otp_requests
        WHERE user_id=$1 AND otp_type='PASSWORD_RESET' ORDER BY created_at DESC LIMIT 1 FOR UPDATE",
    )
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db)
}

pub async fn mark_reset_otp_used(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<(), AppError> {
    sqlx::query("UPDATE otp_requests SET used_at=NOW() WHERE id=$1")
        .bind(id)
        .execute(&mut **tx)
        .await
        .map(|_| ())
        .map_err(map_db)
}

pub async fn update_password(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    hash: &str,
) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET password_hash=$2,updated_at=NOW() WHERE id=$1")
        .bind(user_id)
        .bind(hash)
        .execute(&mut **tx)
        .await
        .map(|_| ())
        .map_err(map_db)
}

fn map_db(error: sqlx::Error) -> AppError {
    match &error {
        sqlx::Error::Database(db) if db.constraint().is_some_and(|c| c == "users_email_unique") => {
            AppError::Conflict("Email already registered".into())
        }
        _ => AppError::Database(error),
    }
}
