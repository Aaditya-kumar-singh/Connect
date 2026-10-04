use crate::{
    config::Config,
    notifications::models::{NotificationPayload, PushError, PushErrorKind},
};
use async_trait::async_trait;
use base64ct::{Base64UrlUnpadded, Encoding};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::Serialize;
use serde_json::json;
use std::{
    fs,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use web_push_native::{
    jwt_simple::algorithms::ES256KeyPair, p256::PublicKey, Auth, WebPushBuilder,
};

fn validate_web_push_endpoint(endpoint: &http::Uri) -> Result<(), PushError> {
    if endpoint.scheme_str() != Some("https") {
        return Err(PushError::Provider {
            kind: PushErrorKind::Permanent,
            message: "Web Push endpoint must use HTTPS".into(),
        });
    }
    let host = endpoint.host().ok_or_else(|| PushError::Provider {
        kind: PushErrorKind::Permanent,
        message: "Web Push endpoint must include a host".into(),
    })?;
    if host.parse::<std::net::IpAddr>().is_ok() {
        return Err(PushError::Provider {
            kind: PushErrorKind::Permanent,
            message: "Web Push endpoint must use a provider hostname".into(),
        });
    }
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    let allowed = [
        "fcm.googleapis.com",
        "push.services.mozilla.com",
        "notify.windows.com",
        "push.apple.com",
        "web.push.apple.com",
    ];
    if !allowed
        .iter()
        .any(|suffix| host == *suffix || host.ends_with(&format!(".{suffix}")))
    {
        return Err(PushError::Provider {
            kind: PushErrorKind::Permanent,
            message: "Web Push endpoint host is not an approved push provider".into(),
        });
    }
    Ok(())
}

#[async_trait]
pub trait PushProvider: Send + Sync {
    async fn send(&self, token: &str, payload: &NotificationPayload) -> Result<(), PushError>;
}

#[derive(Clone, Default)]
pub struct ConsolePushProvider;

#[async_trait]
impl PushProvider for ConsolePushProvider {
    async fn send(&self, token: &str, payload: &NotificationPayload) -> Result<(), PushError> {
        tracing::info!(
            provider = "console",
            token_len = token.len(),
            notification_type = %payload.notification_type,
            title = %payload.title,
            "push notification"
        );
        Ok(())
    }
}

#[derive(Clone)]
pub struct FcmPushProvider {
    client: Client,
    credentials: Arc<FcmCredentials>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct FcmCredentials {
    project_id: String,
    client_email: String,
    private_key: String,
    token_uri: Option<String>,
}

#[derive(Debug, Serialize)]
struct ServiceAccountClaims<'a> {
    iss: &'a str,
    scope: &'a str,
    aud: &'a str,
    iat: u64,
    exp: u64,
}

#[derive(Debug, serde::Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
}

impl FcmPushProvider {
    pub fn from_path(path: &str) -> Result<Self, PushError> {
        let raw = fs::read_to_string(path).map_err(|e| PushError::Provider {
            kind: PushErrorKind::Permanent,
            message: format!("Unable to read FCM credentials: {e}"),
        })?;
        let credentials: FcmCredentials =
            serde_json::from_str(&raw).map_err(|e| PushError::Provider {
                kind: PushErrorKind::Permanent,
                message: format!("Invalid FCM credentials JSON: {e}"),
            })?;
        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .map_err(|e| PushError::Provider {
                    kind: PushErrorKind::Permanent,
                    message: format!("Unable to create FCM client: {e}"),
                })?,
            credentials: Arc::new(credentials),
        })
    }

    pub fn from_json(raw: &str) -> Result<Self, PushError> {
        let credentials: FcmCredentials =
            serde_json::from_str(raw).map_err(|e| PushError::Provider {
                kind: PushErrorKind::Permanent,
                message: format!("Invalid FCM credentials JSON: {e}"),
            })?;
        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .map_err(|e| PushError::Provider {
                    kind: PushErrorKind::Permanent,
                    message: format!("Unable to create FCM client: {e}"),
                })?,
            credentials: Arc::new(credentials),
        })
    }

    async fn access_token(&self) -> Result<String, PushError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let claims = ServiceAccountClaims {
            iss: &self.credentials.client_email,
            scope: "https://www.googleapis.com/auth/firebase.messaging",
            aud: self
                .credentials
                .token_uri
                .as_deref()
                .unwrap_or("https://oauth2.googleapis.com/token"),
            iat: now,
            exp: now + 3600,
        };
        let key =
            EncodingKey::from_rsa_pem(self.credentials.private_key.as_bytes()).map_err(|e| {
                PushError::Provider {
                    kind: PushErrorKind::Permanent,
                    message: format!("Invalid FCM private key: {e}"),
                }
            })?;
        let jwt = encode(&Header::new(Algorithm::RS256), &claims, &key).map_err(|e| {
            PushError::Provider {
                kind: PushErrorKind::Permanent,
                message: format!("Unable to sign FCM credentials: {e}"),
            }
        })?;
        let response = self
            .client
            .post(
                self.credentials
                    .token_uri
                    .as_deref()
                    .unwrap_or("https://oauth2.googleapis.com/token"),
            )
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", jwt.as_str()),
            ])
            .send()
            .await
            .map_err(|e| PushError::Provider {
                kind: if e.is_timeout() {
                    PushErrorKind::Temporary
                } else {
                    PushErrorKind::Permanent
                },
                message: format!("FCM OAuth request failed: {e}"),
            })?;
        if !response.status().is_success() {
            return Err(PushError::Provider {
                kind: if response.status().is_server_error() || response.status().as_u16() == 429 {
                    PushErrorKind::Temporary
                } else {
                    PushErrorKind::Permanent
                },
                message: format!("FCM OAuth returned {}", response.status()),
            });
        }
        response
            .json::<GoogleTokenResponse>()
            .await
            .map(|v| v.access_token)
            .map_err(|e| PushError::Provider {
                kind: PushErrorKind::Permanent,
                message: format!("Invalid FCM OAuth response: {e}"),
            })
    }
}

#[derive(Clone)]
pub struct WebPushProvider {
    client: Client,
    private_key: String,
    subject: String,
}

impl WebPushProvider {
    pub fn from_config(config: &Config) -> Result<Self, PushError> {
        let private_key = config
            .vapid_private_key
            .clone()
            .ok_or_else(|| PushError::Provider {
                kind: PushErrorKind::Permanent,
                message: "VAPID_PRIVATE_KEY is required when PUSH_PROVIDER=webpush".into(),
            })?;
        let subject = config
            .vapid_subject
            .clone()
            .ok_or_else(|| PushError::Provider {
                kind: PushErrorKind::Permanent,
                message: "VAPID_SUBJECT is required when PUSH_PROVIDER=webpush".into(),
            })?;
        if !subject.starts_with("mailto:") && !subject.starts_with("https://") {
            return Err(PushError::Provider {
                kind: PushErrorKind::Permanent,
                message: "VAPID_SUBJECT must be a mailto: or https:// URL".into(),
            });
        }
        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .map_err(|e| PushError::Provider {
                    kind: PushErrorKind::Permanent,
                    message: format!("Unable to create Web Push client: {e}"),
                })?,
            private_key,
            subject,
        })
    }

    async fn send_inner(
        &self,
        token: &str,
        payload: &NotificationPayload,
    ) -> Result<reqwest::Response, PushError> {
        #[derive(serde::Deserialize)]
        struct Subscription {
            endpoint: String,
            keys: SubscriptionKeys,
        }
        #[derive(serde::Deserialize)]
        struct SubscriptionKeys {
            p256dh: String,
            auth: String,
        }

        let subscription: Subscription =
            serde_json::from_str(token).map_err(|e| PushError::Provider {
                kind: PushErrorKind::Permanent,
                message: format!("Invalid Web Push subscription: {e}"),
            })?;
        let endpoint =
            subscription
                .endpoint
                .parse::<http::Uri>()
                .map_err(|e| PushError::Provider {
                    kind: PushErrorKind::Permanent,
                    message: format!("Invalid Web Push endpoint: {e}"),
                })?;
        validate_web_push_endpoint(&endpoint)?;
        let public_key = Base64UrlUnpadded::decode_vec(&subscription.keys.p256dh).map_err(|e| {
            PushError::Provider {
                kind: PushErrorKind::Permanent,
                message: format!("Invalid Web Push p256dh key: {e}"),
            }
        })?;
        let auth = Base64UrlUnpadded::decode_vec(&subscription.keys.auth).map_err(|e| {
            PushError::Provider {
                kind: PushErrorKind::Permanent,
                message: format!("Invalid Web Push auth key: {e}"),
            }
        })?;
        let public_key =
            PublicKey::from_sec1_bytes(&public_key).map_err(|e| PushError::Provider {
                kind: PushErrorKind::Permanent,
                message: format!("Invalid Web Push public key: {e}"),
            })?;
        let auth = Auth::clone_from_slice(&auth);
        let private =
            Base64UrlUnpadded::decode_vec(&self.private_key).map_err(|e| PushError::Provider {
                kind: PushErrorKind::Permanent,
                message: format!("Invalid VAPID private key: {e}"),
            })?;
        let key_pair = ES256KeyPair::from_bytes(&private).map_err(|e| PushError::Provider {
            kind: PushErrorKind::Permanent,
            message: format!("Invalid VAPID key pair: {e}"),
        })?;
        let body = serde_json::to_vec(payload).map_err(|e| PushError::Provider {
            kind: PushErrorKind::Permanent,
            message: format!("Unable to encode Web Push payload: {e}"),
        })?;
        if body.len() > 3052 {
            return Err(PushError::Provider {
                kind: PushErrorKind::Permanent,
                message: "Web Push payload is too large".into(),
            });
        }
        let request = WebPushBuilder::new(endpoint, public_key, auth)
            .with_vapid(&key_pair, &self.subject)
            .build(body)
            .map_err(|e| PushError::Provider {
                kind: PushErrorKind::Permanent,
                message: format!("Unable to build Web Push request: {e}"),
            })?;
        let mut request_builder = self.client.post(request.uri().to_string());
        for (name, value) in request.headers() {
            request_builder = request_builder.header(name.as_str(), value.as_bytes());
        }
        request_builder
            .body(request.into_body())
            .send()
            .await
            .map_err(|e| PushError::Provider {
                kind: if e.is_timeout() || e.is_connect() {
                    PushErrorKind::Temporary
                } else {
                    PushErrorKind::Permanent
                },
                message: format!("Web Push request failed: {e}"),
            })
    }
}

#[async_trait]
impl PushProvider for WebPushProvider {
    async fn send(&self, token: &str, payload: &NotificationPayload) -> Result<(), PushError> {
        let response = self.send_inner(token, payload).await?;
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        if status.as_u16() == 404 || status.as_u16() == 410 {
            return Err(PushError::Provider {
                kind: PushErrorKind::InvalidToken,
                message: "Web Push subscription is no longer valid".into(),
            });
        }
        Err(PushError::Provider {
            kind: if status.as_u16() == 429 || status.is_server_error() {
                PushErrorKind::Temporary
            } else {
                PushErrorKind::Permanent
            },
            message: format!("Web Push returned {status}"),
        })
    }
}

#[async_trait]
impl PushProvider for FcmPushProvider {
    async fn send(&self, token: &str, payload: &NotificationPayload) -> Result<(), PushError> {
        let access_token = self.access_token().await?;
        let data = payload
            .data
            .as_object()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, v.to_string()))
            .collect::<std::collections::HashMap<_, _>>();
        let body = json!({
            "message": {
                "token": token,
                "notification": {
                    "title": payload.title,
                    "body": payload.body.clone().unwrap_or_default()
                },
                "data": data,
                "android": {
                    "priority": if payload.notification_type == "INCOMING_CALL" { "HIGH" } else { "NORMAL" }
                }
            }
        });
        let response = self
            .client
            .post(format!(
                "https://fcm.googleapis.com/v1/projects/{}/messages:send",
                self.credentials.project_id
            ))
            .bearer_auth(access_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| PushError::Provider {
                kind: PushErrorKind::Temporary,
                message: format!("FCM request failed: {e}"),
            })?;
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        let text = response.text().await.unwrap_or_default();
        if text.contains("UNREGISTERED") || status.as_u16() == 404 {
            return Err(PushError::Provider {
                kind: PushErrorKind::InvalidToken,
                message: "FCM token is invalid".into(),
            });
        }
        Err(PushError::Provider {
            kind: if status.is_server_error() || status.as_u16() == 429 {
                PushErrorKind::Temporary
            } else {
                PushErrorKind::Permanent
            },
            message: format!("FCM returned {status}: {text}"),
        })
    }
}

pub fn provider_from_config(config: &Config) -> Result<Arc<dyn PushProvider>, PushError> {
    match config.push_provider.as_str() {
        "console" => Ok(Arc::new(ConsolePushProvider)),
        "fcm" => {
            if let Some(raw) = config.fcm_credentials_json.as_deref() {
                return Ok(Arc::new(FcmPushProvider::from_json(raw)?));
            }
            let path =
                config
                    .fcm_credentials_path
                    .as_deref()
                    .ok_or_else(|| PushError::Provider {
                        kind: PushErrorKind::Permanent,
                        message: "FCM_CREDENTIALS_PATH is required when PUSH_PROVIDER=fcm".into(),
                    })?;
            Ok(Arc::new(FcmPushProvider::from_path(path)?))
        }
        "webpush" => Ok(Arc::new(WebPushProvider::from_config(config)?)),
        other => Err(PushError::Provider {
            kind: PushErrorKind::Permanent,
            message: format!("Unsupported PUSH_PROVIDER: {other}"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_push_endpoint_requires_https() {
        let uri: http::Uri = "http://fcm.googleapis.com/send".parse().unwrap();
        assert!(validate_web_push_endpoint(&uri).is_err());
    }

    #[test]
    fn web_push_endpoint_rejects_ip_literals() {
        let uri: http::Uri = "https://127.0.0.1/send".parse().unwrap();
        assert!(validate_web_push_endpoint(&uri).is_err());
    }

    #[test]
    fn web_push_endpoint_rejects_unapproved_hosts() {
        let uri: http::Uri = "https://example.com/send".parse().unwrap();
        assert!(validate_web_push_endpoint(&uri).is_err());
    }

    #[test]
    fn web_push_endpoint_accepts_known_provider_host() {
        let uri: http::Uri = "https://fcm.googleapis.com/send".parse().unwrap();
        assert!(validate_web_push_endpoint(&uri).is_ok());
    }
}
