use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub rust_log: String,
    pub environment: String,
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_min_connections: u32,
    pub redis_url: String,
    pub redis_max_connections: u32,
    pub access_token_secret: String,
    pub refresh_token_secret: String,
    pub access_token_expiry_seconds: u64,
    pub refresh_token_expiry_days: u64,
    pub argon2_memory_kb: u32,
    pub argon2_iterations: u32,
    pub argon2_parallelism: u32,
    pub email_provider: String,
    pub email_from: String,
    pub cors_allowed_origins: String,
    pub trust_proxy_headers: bool,
    pub r2_enabled: bool,
    pub r2_access_key_id: String,
    pub r2_secret_access_key: String,
    pub r2_bucket_name: String,
    pub r2_endpoint: String,
    pub push_provider: String,
    pub fcm_credentials_path: Option<String>,
    pub fcm_credentials_json: Option<String>,
    pub vapid_private_key: Option<String>,
    pub vapid_public_key: Option<String>,
    pub vapid_subject: Option<String>,
    pub turn_server_url: Option<String>,
    pub turn_secret: Option<String>,
    pub turn_realm: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let server_port = env::var("PORT")
            .or_else(|_| env::var("SERVER_PORT"))
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|e| format!("Invalid PORT/SERVER_PORT: {}", e))?;

        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "ybm_connect=info".to_string());
        let environment = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ybm:ybm_dev_password@localhost:5432/ybm_connect".to_string()
        });

        let database_max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .map_err(|e| format!("Invalid DATABASE_MAX_CONNECTIONS: {}", e))?;

        let database_min_connections = env::var("DATABASE_MIN_CONNECTIONS")
            .unwrap_or_else(|_| "2".to_string())
            .parse::<u32>()
            .map_err(|e| format!("Invalid DATABASE_MIN_CONNECTIONS: {}", e))?;

        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let redis_max_connections = env::var("REDIS_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .map_err(|e| format!("Invalid REDIS_MAX_CONNECTIONS: {}", e))?;

        let access_token_secret = env::var("ACCESS_TOKEN_SECRET")
            .unwrap_or_else(|_| "dev-access-token-secret-change-in-production".to_string());

        let refresh_token_secret = env::var("REFRESH_TOKEN_SECRET")
            .unwrap_or_else(|_| "dev-refresh-token-secret-change-in-production".to_string());

        let access_token_expiry_seconds = env::var("ACCESS_TOKEN_EXPIRY_SECONDS")
            .unwrap_or_else(|_| "900".to_string())
            .parse::<u64>()
            .map_err(|e| format!("Invalid ACCESS_TOKEN_EXPIRY_SECONDS: {}", e))?;

        let refresh_token_expiry_days = env::var("REFRESH_TOKEN_EXPIRY_DAYS")
            .unwrap_or_else(|_| "30".to_string())
            .parse::<u64>()
            .map_err(|e| format!("Invalid REFRESH_TOKEN_EXPIRY_DAYS: {}", e))?;

        let argon2_memory_kb = env::var("ARGON2_MEMORY_KB")
            .unwrap_or_else(|_| "65536".to_string())
            .parse::<u32>()
            .map_err(|e| format!("Invalid ARGON2_MEMORY_KB: {}", e))?;

        let argon2_iterations = env::var("ARGON2_ITERATIONS")
            .unwrap_or_else(|_| "3".to_string())
            .parse::<u32>()
            .map_err(|e| format!("Invalid ARGON2_ITERATIONS: {}", e))?;

        let argon2_parallelism = env::var("ARGON2_PARALLELISM")
            .unwrap_or_else(|_| "4".to_string())
            .parse::<u32>()
            .map_err(|e| format!("Invalid ARGON2_PARALLELISM: {}", e))?;

        let email_provider = env::var("EMAIL_PROVIDER").unwrap_or_else(|_| "console".to_string());
        let email_from = env::var("EMAIL_FROM").unwrap_or_else(|_| "noreply@localhost".to_string());
        let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000,http://localhost:19006".to_string());
        let trust_proxy_headers = env::var("TRUST_PROXY_HEADERS")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .map_err(|e| format!("Invalid TRUST_PROXY_HEADERS: {e}"))?;
        let r2_enabled = env::var("R2_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .map_err(|e| format!("Invalid R2_ENABLED: {e}"))?;
        let r2_access_key_id =
            env::var("R2_ACCESS_KEY_ID").unwrap_or_else(|_| "dev-access-key".to_string());
        let r2_secret_access_key =
            env::var("R2_SECRET_ACCESS_KEY").unwrap_or_else(|_| "dev-secret-key".to_string());
        let r2_bucket_name =
            env::var("R2_BUCKET_NAME").unwrap_or_else(|_| "ybm-connect-dev".to_string());
        let r2_endpoint =
            env::var("R2_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string());
        let push_provider = env::var("PUSH_PROVIDER").unwrap_or_else(|_| "console".to_string());
        let fcm_credentials_path = env::var("FCM_CREDENTIALS_PATH").ok();
        let fcm_credentials_json = env::var("FCM_CREDENTIALS_JSON").ok();
        let vapid_private_key = env::var("VAPID_PRIVATE_KEY").ok();
        let vapid_public_key = env::var("VAPID_PUBLIC_KEY").ok();
        let vapid_subject = env::var("VAPID_SUBJECT").ok();
        let turn_server_url = env::var("TURN_SERVER_URL").ok();
        let turn_secret = env::var("TURN_SECRET").ok();
        let turn_realm = env::var("TURN_REALM").ok();

        if environment.eq_ignore_ascii_case("production") {
            let insecure_defaults = [
                (
                    "ACCESS_TOKEN_SECRET",
                    &access_token_secret,
                    "dev-access-token-secret-change-in-production",
                ),
                (
                    "REFRESH_TOKEN_SECRET",
                    &refresh_token_secret,
                    "dev-refresh-token-secret-change-in-production",
                ),
            ];
            if insecure_defaults
                .iter()
                .any(|(_, value, default)| value == default)
            {
                return Err(
                    "Production configuration contains an insecure development secret".to_string(),
                );
            }
            if cors_allowed_origins.contains("localhost") {
                return Err("Production CORS origins cannot include localhost".to_string());
            }
            if database_url.contains("localhost") || database_url.contains("127.0.0.1") {
                return Err("Production DATABASE_URL cannot point to localhost".to_string());
            }
            if redis_url.contains("localhost") || redis_url.contains("127.0.0.1") {
                return Err("Production REDIS_URL cannot point to localhost".to_string());
            }
            if r2_enabled {
                if r2_access_key_id == "dev-access-key" || r2_secret_access_key == "dev-secret-key"
                {
                    return Err(
                        "Production R2 configuration contains an insecure development credential"
                            .to_string(),
                    );
                }
                if r2_endpoint.contains("localhost") || r2_endpoint.contains("127.0.0.1") {
                    return Err("Production R2_ENDPOINT cannot point to localhost".to_string());
                }
            }
            if push_provider.eq_ignore_ascii_case("console") {
                return Err("Production PUSH_PROVIDER cannot be console".to_string());
            }
        }

        if database_max_connections == 0 || database_min_connections == 0 {
            return Err("Database connection limits must be greater than zero".to_string());
        }
        if database_min_connections > database_max_connections {
            return Err(
                "DATABASE_MIN_CONNECTIONS cannot exceed DATABASE_MAX_CONNECTIONS".to_string(),
            );
        }
        if redis_max_connections == 0 {
            return Err("REDIS_MAX_CONNECTIONS must be greater than zero".to_string());
        }
        if argon2_memory_kb == 0 || argon2_iterations == 0 || argon2_parallelism == 0 {
            return Err("Argon2 parameters must be greater than zero".to_string());
        }

        Ok(Self {
            server_host,
            server_port,
            rust_log,
            environment,
            database_url,
            database_max_connections,
            database_min_connections,
            redis_url,
            redis_max_connections,
            access_token_secret,
            refresh_token_secret,
            access_token_expiry_seconds,
            refresh_token_expiry_days,
            argon2_memory_kb,
            argon2_iterations,
            argon2_parallelism,
            email_provider,
            email_from,
            cors_allowed_origins,
            trust_proxy_headers,
            r2_enabled,
            r2_access_key_id,
            r2_secret_access_key,
            r2_bucket_name,
            r2_endpoint,
            push_provider,
            fcm_credentials_path,
            fcm_credentials_json,
            vapid_private_key,
            vapid_public_key,
            vapid_subject,
            turn_server_url,
            turn_secret,
            turn_realm,
        })
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
