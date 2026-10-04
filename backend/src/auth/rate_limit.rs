use crate::{app_state::AppState, domain::errors::AppError};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

pub async fn middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path = request.uri().path();
    let (limit, window) = match path {
        "/api/v1/auth/register" => (5, 3600),
        "/api/v1/auth/verify-email" => (5, 600),
        "/api/v1/auth/login" => (30, 900),
        "/api/v1/auth/refresh" => (60, 900),
        "/api/v1/auth/logout" | "/api/v1/auth/logout-all" => (60, 900),
        "/api/v1/auth/forgot-password" => (20, 3600),
        "/api/v1/auth/reset-password" => (20, 3600),
        _ => return Ok(next.run(request).await),
    };
    // X-Forwarded-For is only used when the deployment explicitly trusts a
    // reverse proxy. Otherwise the caller cannot choose the rate-limit identity.
    let identity = if state.config().trust_proxy_headers {
        request
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .unwrap_or("unknown")
    } else {
        "direct"
    };
    let key = format!("rate:auth:{path}:{identity}");
    let mut conn = state.redis().get_multiplexed_async_connection().await?;
    let count: i64 = redis::cmd("INCR").arg(&key).query_async(&mut conn).await?;
    if count == 1 {
        let _: () = redis::cmd("EXPIRE")
            .arg(&key)
            .arg(window)
            .query_async(&mut conn)
            .await?;
    }
    if count > limit {
        return Err(AppError::RateLimited);
    }
    Ok(next.run(request).await)
}

pub async fn check_key(
    state: &AppState,
    key: &str,
    limit: i64,
    window: usize,
) -> Result<(), AppError> {
    let mut conn = state.redis().get_multiplexed_async_connection().await?;
    let count: i64 = redis::cmd("INCR").arg(key).query_async(&mut conn).await?;
    if count == 1 {
        let _: () = redis::cmd("EXPIRE")
            .arg(key)
            .arg(window)
            .query_async(&mut conn)
            .await?;
    }
    if count > limit {
        return Err(AppError::RateLimited);
    }
    Ok(())
}
