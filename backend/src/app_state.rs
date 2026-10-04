use crate::{
    config::Config, infrastructure::metrics::Metrics, notifications::worker::NotificationSender,
    websocket::manager::ConnectionManager,
};
use redis::Client;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pub config: Config,
    pub db_pool: PgPool,
    pub redis: Client,
    pub storage: aws_sdk_s3::Client,
    pub connection_manager: ConnectionManager,
    pub instance_id: Uuid,
    pub notification_tx: NotificationSender,
    pub metrics: Arc<Metrics>,
}

impl AppState {
    pub fn new(config: Config, db_pool: PgPool, redis: Client) -> Self {
        let storage_config = aws_sdk_s3::config::Builder::new()
            .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
            .region(aws_sdk_s3::config::Region::new("auto"))
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                &config.r2_access_key_id,
                &config.r2_secret_access_key,
                None,
                None,
                "ybm-connect-r2",
            ))
            .endpoint_url(&config.r2_endpoint)
            .force_path_style(true)
            .build();
        let storage = aws_sdk_s3::Client::from_conf(storage_config);
        let notification_tx = crate::notifications::worker::start(db_pool.clone(), config.clone());
        let metrics =
            Arc::new(Metrics::new().expect("metrics registry initialization must succeed"));

        Self {
            inner: Arc::new(AppStateInner {
                config,
                db_pool,
                redis,
                storage,
                connection_manager: ConnectionManager::default(),
                instance_id: Uuid::new_v4(),
                notification_tx,
                metrics,
            }),
        }
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn db(&self) -> &PgPool {
        &self.inner.db_pool
    }

    pub fn redis(&self) -> &Client {
        &self.inner.redis
    }

    pub fn storage(&self) -> &aws_sdk_s3::Client {
        &self.inner.storage
    }

    pub fn connection_manager(&self) -> &ConnectionManager {
        &self.inner.connection_manager
    }

    pub fn instance_id(&self) -> Uuid {
        self.inner.instance_id
    }

    pub fn notification_sender(&self) -> &NotificationSender {
        &self.inner.notification_tx
    }

    pub fn metrics(&self) -> &Metrics {
        &self.inner.metrics
    }
}
