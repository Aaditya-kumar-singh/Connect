use crate::notifications::{
    models::{NotificationJob, PushErrorKind},
    provider::provider_from_config,
    repository,
};
use tokio::sync::mpsc;

pub type NotificationSender = mpsc::Sender<NotificationJob>;

pub fn start(db: sqlx::PgPool, config: crate::config::Config) -> NotificationSender {
    let (tx, mut rx) = mpsc::channel::<NotificationJob>(256);
    tokio::spawn(async move {
        let provider = match provider_from_config(&config) {
            Ok(provider) => provider,
            Err(error) => {
                tracing::error!(error = %error, "notification provider initialization failed");
                return;
            }
        };
        while let Some(job) = rx.recv().await {
            let targets = match repository::targets(&db, job.user_id).await {
                Ok(targets) => targets,
                Err(error) => {
                    tracing::error!(error = %error, user_id = %job.user_id, "notification target lookup failed");
                    continue;
                }
            };
            let mut delivered = false;
            for target in targets {
                if config.push_provider != "console"
                    && !target.provider.eq_ignore_ascii_case(&config.push_provider)
                    && !(config.push_provider == "fcm"
                        && target.provider.eq_ignore_ascii_case("FCM"))
                    && !(config.push_provider == "webpush"
                        && target.provider.eq_ignore_ascii_case("WEB_PUSH"))
                {
                    continue;
                }
                let mut sent = false;
                for attempt in 0..=3 {
                    match provider.send(&target.token, &job.payload).await {
                        Ok(()) => {
                            sent = true;
                            break;
                        }
                        Err(error) if error.kind() == PushErrorKind::InvalidToken => {
                            let _ = repository::remove_token(&db, target.device_id).await;
                            break;
                        }
                        Err(error) if error.kind() == PushErrorKind::Temporary && attempt < 3 => {
                            tokio::time::sleep(std::time::Duration::from_millis(
                                250 * (1 << attempt),
                            ))
                            .await;
                        }
                        Err(error) => {
                            tracing::warn!(error = %error, user_id = %job.user_id, "push delivery failed");
                            break;
                        }
                    }
                }
                delivered |= sent;
            }
            if delivered {
                if let Err(error) = repository::mark_sent(&db, job.notification_id).await {
                    tracing::error!(error = %error, notification_id = %job.notification_id, "notification audit update failed");
                }
            }
        }
    });
    tx
}
