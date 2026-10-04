use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub fn create_pool(
    database_url: &str,
    max_connections: u32,
    min_connections: u32,
) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .connect_lazy(database_url)
}

pub async fn check_health(pool: &PgPool) -> Result<Duration, sqlx::Error> {
    let start = std::time::Instant::now();
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(start.elapsed())
}
