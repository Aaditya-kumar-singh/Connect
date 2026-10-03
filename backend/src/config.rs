use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub rust_log: String,
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
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|e| format!("Invalid SERVER_PORT: {}", e))?;

        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "ybm_connect=info".to_string());

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://ybm:ybm_dev_password@localhost:5432/ybm_connect".to_string());

        let database_max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .map_err(|e| format!("Invalid DATABASE_MAX_CONNECTIONS: {}", e))?;

        let database_min_connections = env::var("DATABASE_MIN_CONNECTIONS")
            .unwrap_or_else(|_| "2".to_string())
            .parse::<u32>()
            .map_err(|e| format!("Invalid DATABASE_MIN_CONNECTIONS: {}", e))?;

        let redis_url = env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

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

        Ok(Self {
            server_host,
            server_port,
            rust_log,
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
        })
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
