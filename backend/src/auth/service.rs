use crate::{
    app_state::AppState,
    auth::{email::EmailSender, models::*, repository, tokens, validation},
    domain::errors::AppError,
};
use argon2::password_hash::rand_core::OsRng;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

fn argon2() -> Result<Argon2<'static>, AppError> {
    let params = Params::new(19456, 2, 1, None)
        .map_err(|e| AppError::Internal(format!("Argon2 configuration: {e}")))?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        argon2()?
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| AppError::Internal(format!("Password hashing failed: {e}")))
    })
    .await
    .map_err(|e| AppError::Internal(format!("Password hashing task failed: {e}")))?
}

async fn verify_password(password: String, stored: String) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || {
        let parsed = PasswordHash::new(&stored)
            .map_err(|e| AppError::Internal(format!("Invalid password hash: {e}")))?;
        Ok(argon2()?
            .verify_password(password.as_bytes(), &parsed)
            .is_ok())
    })
    .await
    .map_err(|e| AppError::Internal(format!("Password verification task failed: {e}")))?
}

fn otp_hash(otp: &str) -> String {
    let digest = Sha256::digest(otp.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn generic_credentials_error() -> AppError {
    AppError::Unauthorized("Invalid email or password".into())
}

pub async fn register(
    state: &AppState,
    input: RegisterRequest,
    email: &impl EmailSender,
) -> Result<(), AppError> {
    let email_address = validation::normalize_email(&input.email)?;
    validation::validate_password(&input.password)?;
    validation::validate_display_name(&input.display_name)?;
    let password_hash = hash_password(input.password).await?;
    let otp = tokens::random_otp();
    let mut tx = state.db().begin().await?;
    let user = repository::create_user(
        &mut tx,
        &email_address,
        &password_hash,
        input.display_name.trim(),
    )
    .await?;
    repository::create_email_verification(
        &mut tx,
        user.id,
        &otp_hash(&otp),
        Utc::now() + Duration::minutes(10),
    )
    .await?;
    tx.commit().await?;
    email
        .send_otp(&email_address, &otp, "email_verification")
        .await
}

pub async fn verify_email(state: &AppState, input: VerifyEmailRequest) -> Result<(), AppError> {
    let email_address = validation::normalize_email(&input.email)?;
    validation::validate_otp(&input.otp)?;
    let user = repository::find_user_by_email(state.db(), &email_address)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid OTP".into()))?;
    let mut tx = state.db().begin().await?;
    repository::verify_email(&mut tx, user.id, &otp_hash(&input.otp)).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn login(state: &AppState, input: LoginRequest) -> Result<TokenResponse, AppError> {
    let email_address = validation::normalize_email(&input.email)?;
    crate::auth::rate_limit::check_key(state, &format!("rate:auth:login:{email_address}"), 10, 900)
        .await?;
    validation::validate_device_type(&input.device_type)?;
    let user = repository::find_user_by_email(state.db(), &email_address)
        .await?
        .ok_or_else(generic_credentials_error)?;
    if user.status == "PENDING_VERIFICATION" {
        return Err(AppError::Forbidden("Email verification required".into()));
    }
    if user.status != "ACTIVE" {
        return Err(AppError::Forbidden("Account unavailable".into()));
    }
    if !verify_password(input.password, user.password_hash.clone()).await? {
        tracing::warn!(
            event = "auth.login_failed",
            reason = "invalid_credentials",
            "Authentication failed"
        );
        return Err(generic_credentials_error());
    }

    let access = tokens::access_token();
    let refresh = tokens::refresh_token();
    let access_hash = tokens::hash_token(&access);
    let refresh_hash = tokens::hash_token(&refresh);
    let family = tokens::random_family_id();
    let now = Utc::now();
    let refresh_exp = now + Duration::days(state.config().refresh_token_expiry_days as i64);
    let mut tx = state.db().begin().await?;
    repository::upsert_device(
        &mut tx,
        user.id,
        input.device_id,
        input.device_name.as_deref(),
        &input.device_type,
    )
    .await?;
    let session_id =
        repository::create_session(&mut tx, user.id, input.device_id, refresh_exp).await?;
    repository::create_refresh_token(
        &mut tx,
        user.id,
        session_id,
        &refresh_hash,
        family,
        refresh_exp,
    )
    .await?;
    tx.commit().await?;
    store_access_token(
        state,
        &access_hash,
        user.id,
        session_id,
        input.device_id,
        state.config().access_token_expiry_seconds,
    )
    .await?;
    Ok(TokenResponse {
        access_token: access,
        refresh_token: refresh,
        access_token_expires_in: state.config().access_token_expiry_seconds,
        refresh_token_expires_in: state.config().refresh_token_expiry_days * 86400,
        session_id,
        device_id: input.device_id,
    })
}

pub async fn refresh(state: &AppState, input: RefreshRequest) -> Result<TokenResponse, AppError> {
    let old_hash = tokens::hash_token(&input.refresh_token);
    let record = repository::find_refresh_token(state.db(), &old_hash)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid refresh token".into()))?;
    if record.revoked_at.is_some() {
        let mut tx = state.db().begin().await?;
        repository::revoke_refresh_family(&mut tx, record.family_id).await?;
        repository::revoke_session(&mut tx, record.session_id).await?;
        tx.commit().await?;
        revoke_session_tokens(state, record.session_id).await?;
        tracing::warn!(
            event = "auth.refresh_reuse_detected",
            "Refresh-token reuse detected; session family revoked"
        );
        return Err(AppError::Unauthorized("Invalid refresh token".into()));
    }
    if record.expires_at <= Utc::now() {
        return Err(AppError::Unauthorized("Invalid refresh token".into()));
    }

    let new_access = tokens::access_token();
    let new_refresh = tokens::refresh_token();
    let new_refresh_hash = tokens::hash_token(&new_refresh);
    let mut tx = state.db().begin().await?;
    let rotated = repository::rotate_refresh_token(
        &mut tx,
        record.id,
        &new_refresh_hash,
        Utc::now() + Duration::days(state.config().refresh_token_expiry_days as i64),
    )
    .await?;
    tx.commit().await?;
    if !rotated {
        let mut reuse_tx = state.db().begin().await?;
        repository::revoke_refresh_family(&mut reuse_tx, record.family_id).await?;
        repository::revoke_session(&mut reuse_tx, record.session_id).await?;
        reuse_tx.commit().await?;
        revoke_session_tokens(state, record.session_id).await?;
        tracing::warn!(
            event = "auth.refresh_concurrent_reuse",
            "Concurrent refresh-token reuse detected; session family revoked"
        );
        return Err(AppError::Unauthorized("Invalid refresh token".into()));
    }
    let device_id = repository::session_device_id(state.db(), record.session_id).await?;

    store_access_token(
        state,
        &tokens::hash_token(&new_access),
        record.user_id,
        record.session_id,
        device_id,
        state.config().access_token_expiry_seconds,
    )
    .await?;
    Ok(TokenResponse {
        access_token: new_access,
        refresh_token: new_refresh,
        access_token_expires_in: state.config().access_token_expiry_seconds,
        refresh_token_expires_in: state.config().refresh_token_expiry_days * 86400,
        session_id: record.session_id,
        device_id,
    })
}

pub async fn revoke_session_by_id(state: &AppState, session_id: Uuid) -> Result<(), AppError> {
    let mut tx = state.db().begin().await?;
    repository::revoke_session(&mut tx, session_id).await?;
    tx.commit().await?;
    revoke_session_tokens(state, session_id).await?;
    Ok(())
}

pub async fn logout(state: &AppState, auth: AuthenticatedSession) -> Result<(), AppError> {
    let mut tx = state.db().begin().await?;
    repository::revoke_session(&mut tx, auth.session_id).await?;
    tx.commit().await?;
    revoke_session_tokens(state, auth.session_id).await?;
    Ok(())
}

pub async fn logout_all(state: &AppState, auth: AuthenticatedSession) -> Result<(), AppError> {
    let session_ids = repository::active_session_ids(state.db(), auth.user_id).await?;
    let mut tx = state.db().begin().await?;
    repository::revoke_all_sessions(&mut tx, auth.user_id).await?;
    tx.commit().await?;
    for session_id in session_ids {
        revoke_session_tokens(state, session_id).await?;
    }
    Ok(())
}

pub async fn forgot_password(
    state: &AppState,
    input: ForgotPasswordRequest,
    email: &impl EmailSender,
) -> Result<(), AppError> {
    let email_address = validation::normalize_email(&input.email)?;
    crate::auth::rate_limit::check_key(
        state,
        &format!("rate:auth:forgot:{email_address}"),
        3,
        3600,
    )
    .await?;
    if let Some(user) = repository::find_user_by_email(state.db(), &email_address).await? {
        let otp = tokens::random_otp();
        let mut tx = state.db().begin().await?;
        repository::create_reset_otp(
            &mut tx,
            user.id,
            &otp_hash(&otp),
            Utc::now() + Duration::minutes(10),
        )
        .await?;
        tx.commit().await?;
        email
            .send_otp(&email_address, &otp, "password_reset")
            .await?;
    }
    Ok(())
}

pub async fn reset_password(state: &AppState, input: ResetPasswordRequest) -> Result<(), AppError> {
    let email_address = validation::normalize_email(&input.email)?;
    validation::validate_otp(&input.otp)?;
    validation::validate_password(&input.new_password)?;
    let user = repository::find_user_by_email(state.db(), &email_address)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid OTP".into()))?;
    let new_hash = hash_password(input.new_password).await?;
    let session_ids = repository::active_session_ids(state.db(), user.id).await?;
    let mut tx = state.db().begin().await?;
    let row = repository::find_reset_otp(&mut tx, user.id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid OTP".into()))?;
    if row.4.is_some() {
        return Err(AppError::Unauthorized("Invalid OTP".into()));
    }
    if row.3 <= Utc::now() {
        return Err(AppError::Gone("OTP expired".into()));
    }
    if row.2 >= 5 {
        return Err(AppError::RateLimited);
    }
    if row.1 != otp_hash(&input.otp) {
        sqlx::query("UPDATE otp_requests SET attempts=attempts+1 WHERE id=$1")
            .bind(row.0)
            .execute(&mut *tx)
            .await?;
        return Err(AppError::Unauthorized("Invalid OTP".into()));
    }
    repository::mark_reset_otp_used(&mut tx, row.0).await?;
    repository::update_password(&mut tx, user.id, &new_hash).await?;
    repository::revoke_all_sessions(&mut tx, user.id).await?;
    tx.commit().await?;
    for session_id in session_ids {
        revoke_session_tokens(state, session_id).await?;
    }
    Ok(())
}

async fn store_access_token(
    state: &AppState,
    hash: &str,
    user_id: Uuid,
    session_id: Uuid,
    device_id: Uuid,
    ttl: u64,
) -> Result<(), AppError> {
    let mut conn = state.redis().get_multiplexed_async_connection().await?;
    let key = format!("session:{hash}");
    let value =
        serde_json::json!({"user_id": user_id, "session_id": session_id, "device_id": device_id})
            .to_string();
    let _: () = redis::cmd("SET")
        .arg(key)
        .arg(value)
        .arg("EX")
        .arg(ttl)
        .query_async(&mut conn)
        .await?;
    let set_key = format!("session_tokens:{session_id}");
    let _: () = redis::cmd("SADD")
        .arg(&set_key)
        .arg(hash)
        .query_async(&mut conn)
        .await?;
    let _: () = redis::cmd("EXPIRE")
        .arg(&set_key)
        .arg(ttl)
        .query_async(&mut conn)
        .await?;
    Ok(())
}

pub async fn authenticate(state: &AppState, token: &str) -> Result<AuthenticatedSession, AppError> {
    let hash = tokens::hash_token(token);
    let mut conn = state.redis().get_multiplexed_async_connection().await?;
    let key = format!("session:{hash}");
    let value: Option<String> = redis::cmd("GET").arg(key).query_async(&mut conn).await?;
    let value =
        value.ok_or_else(|| AppError::Unauthorized("Invalid or expired access token".into()))?;
    serde_json::from_str::<AuthenticatedSessionWire>(&value)
        .map(|v| AuthenticatedSession {
            user_id: v.user_id,
            session_id: v.session_id,
            device_id: v.device_id,
            access_token_hash: hash,
        })
        .map_err(|_| AppError::Unauthorized("Invalid or expired access token".into()))
}

#[derive(serde::Deserialize)]
struct AuthenticatedSessionWire {
    user_id: Uuid,
    session_id: Uuid,
    device_id: Uuid,
}

async fn revoke_session_tokens(state: &AppState, session_id: Uuid) -> Result<(), AppError> {
    let mut conn = state.redis().get_multiplexed_async_connection().await?;
    let set_key = format!("session_tokens:{session_id}");
    let hashes: Vec<String> = redis::cmd("SMEMBERS")
        .arg(&set_key)
        .query_async(&mut conn)
        .await?;
    for hash in hashes {
        let key = format!("session:{hash}");
        let _: () = redis::cmd("DEL").arg(key).query_async(&mut conn).await?;
    }
    let _: () = redis::cmd("DEL")
        .arg(set_key)
        .query_async(&mut conn)
        .await?;
    Ok(())
}
