use crate::config::Config;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pub config: Config,
    pub db_pool: PgPool,
    pub redis: ConnectionManager,
    pub start_time: Instant,
}

impl AppState {
    pub fn new(config: Config, db_pool: PgPool, redis: ConnectionManager) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                config,
                db_pool,
                redis,
                start_time: Instant::now(),
            }),
        }
    }
}
