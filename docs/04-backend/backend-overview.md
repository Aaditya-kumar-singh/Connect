# Backend Overview — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-BE-001`                               |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-ARCH-001, DOC-BE-002 through 009      |
| **Related ADRs**  | ADR-001, ADR-002, ADR-003                  |

---

## 1. Rust Architecture

### Crate Structure

The backend is a single Cargo binary crate. No workspace (unnecessary complexity at this scale).

```toml
# backend/Cargo.toml
[package]
name = "ybm-connect"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web framework
axum = { version = "0.7", features = ["ws", "multipart"] }
axum-extra = { version = "0.9", features = ["typed-header"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace", "timeout", "compression-gzip"] }

# Async runtime
tokio = { version = "1", features = ["full"] }
tokio-util = { version = "0.7", features = ["rt"] }

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "postgres", "uuid", "chrono", "json"] }

# Redis
redis = { version = "0.25", features = ["tokio-comp", "connection-manager"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Authentication
argon2 = "0.5"
rand = "0.8"
sha2 = "0.10"
hmac = "0.12"
base64 = "0.22"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Utilities
uuid = { version = "1", features = ["v4", "v7", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
dotenvy = "0.15"
validator = { version = "0.18", features = ["derive"] }

# Object storage (S3-compatible for R2)
aws-sdk-s3 = "1"
aws-config = "1"
```

### Module Organization

```
src/
├── main.rs                    # Entry point: config load, build app, run server
├── config.rs                  # Config struct parsed from env vars
├── app_state.rs               # AppState: shared across handlers (DB pool, Redis, config)
├── router.rs                  # Axum router construction
│
├── domain/                    # Pure data types, enums, validation rules
│   ├── mod.rs
│   ├── user.rs                # User, UserStatus, CreateUserRequest
│   ├── message.rs             # Message, MessageStatus, ContentType
│   ├── conversation.rs        # Conversation, ConversationType
│   ├── group.rs               # Group, GroupRole
│   ├── call.rs                # Call, CallStatus, CallType
│   ├── media.rs               # MediaObject, MediaType
│   └── errors.rs              # AppError enum, error conversion
│
├── auth/                      # Authentication module
│   ├── mod.rs                 # Module exports
│   ├── handlers.rs            # Axum route handlers (register, login, etc.)
│   ├── service.rs             # AuthService: business logic
│   ├── repository.rs          # UserRepository, SessionRepository: DB queries
│   └── errors.rs              # AuthError enum
│
├── messaging/                 # Messaging module
│   ├── mod.rs
│   ├── handlers.rs            # REST handlers (forward message)
│   ├── ws_handlers.rs         # WebSocket event handlers
│   ├── service.rs             # MessagingService: send, edit, delete, react
│   ├── repository.rs          # MessageRepository: CRUD, idempotency
│   └── errors.rs
│
├── groups/                    # Groups module
│   ├── mod.rs
│   ├── handlers.rs
│   ├── service.rs
│   ├── repository.rs
│   └── errors.rs
│
├── media/                     # Media module
│   ├── mod.rs
│   ├── handlers.rs            # Upload/download handlers
│   ├── service.rs             # Processing, validation, thumbnail generation
│   ├── repository.rs          # MediaRepository
│   └── processor.rs           # Image/video processing
│
├── calls/                     # Call signaling module
│   ├── mod.rs
│   ├── ws_handlers.rs         # WebSocket call event handlers
│   ├── service.rs             # CallService: state machine
│   ├── repository.rs          # CallRepository
│   └── state_machine.rs       # Call state transitions
│
├── presence/                  # Presence module
│   ├── mod.rs
│   ├── service.rs             # PresenceService: track, broadcast
│   └── typing.rs              # Typing indicator logic
│
├── notifications/             # Push notifications
│   ├── mod.rs
│   ├── service.rs             # NotificationService
│   └── providers/
│       ├── mod.rs
│       ├── console.rs         # Dev: log to console
│       ├── fcm.rs             # Firebase Cloud Messaging
│       └── web_push.rs        # W3C Web Push
│
├── websocket/                 # WebSocket connection management
│   ├── mod.rs
│   ├── connection.rs          # Individual connection handling
│   ├── manager.rs             # Connection registry
│   ├── protocol.rs            # Message framing, parsing
│   └── router.rs              # Event type → handler routing
│
├── workers/                   # Background workers
│   ├── mod.rs
│   ├── supervisor.rs          # Worker supervisor
│   ├── message_worker.rs      # Message processing
│   ├── presence_worker.rs     # Presence management
│   ├── notification_worker.rs # Push notification sending
│   ├── cleanup_worker.rs      # Expired data cleanup
│   └── health_worker.rs       # Dependency health checking
│
├── middleware/                # Axum middleware
│   ├── mod.rs
│   ├── auth.rs                # Token extraction and validation
│   ├── rate_limit.rs          # Rate limiting middleware
│   ├── request_id.rs          # Generate/propagate request ID
│   └── logging.rs             # Request/response logging
│
└── infrastructure/            # External system adapters
    ├── mod.rs
    ├── database.rs            # PostgreSQL pool setup
    ├── redis_client.rs        # Redis connection and helpers
    ├── redis_pubsub.rs        # Redis pub/sub subscriber
    ├── r2.rs                  # Cloudflare R2 (S3 SDK)
    ├── email.rs               # Email sending (trait + implementations)
    └── push.rs                # Push notification (trait + implementations)
```

## 2. Axum Architecture

### Application Setup (main.rs pattern)

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load configuration
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;
    
    // 2. Initialize tracing
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(&config.rust_log)
        .init();
    
    // 3. Connect to infrastructure
    let db_pool = database::create_pool(&config.database_url).await?;
    let redis = redis_client::create_pool(&config.redis_url).await?;
    let r2_client = r2::create_client(&config).await?;
    
    // 4. Run migrations
    sqlx::migrate!("./migrations").run(&db_pool).await?;
    
    // 5. Build application state
    let state = AppState::new(config, db_pool, redis, r2_client);
    
    // 6. Build router
    let app = router::build(state.clone());
    
    // 7. Start background workers
    let supervisor = Supervisor::new(state.clone());
    let supervisor_handle = tokio::spawn(supervisor.run());
    
    // 8. Start server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(&state.config.server_addr()).await?;
    tracing::info!(addr = %state.config.server_addr(), "Server starting");
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    // 9. Shutdown workers
    supervisor_handle.abort();
    tracing::info!("Server stopped");
    Ok(())
}
```

### Error Handling Pattern

```rust
// Domain error type
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Rate limited")]
    RateLimited,
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
}

// Convert to Axum response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AppError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            AppError::Forbidden(_) => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            AppError::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT"),
            AppError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, "VALIDATION_ERROR"),
            AppError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED"),
            AppError::Internal(_) | AppError::Database(_) | AppError::Redis(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR")
            }
        };
        
        let body = json!({
            "success": false,
            "data": null,
            "error": {
                "code": code,
                "message": self.to_string(),
            }
        });
        
        (status, Json(body)).into_response()
    }
}
```

## 3. Tokio Concurrency Model

### Task Types

| Task Type | Executor | When to Use | Example |
|-----------|----------|-------------|---------|
| `tokio::spawn` | Async executor (green thread) | I/O-bound work, WebSocket handlers | Message routing, DB queries |
| `tokio::task::spawn_blocking` | Blocking threadpool | CPU-bound or blocking work | Argon2id hashing, image thumbnails |
| `tokio::spawn` with `select!` | Async executor | Waiting on multiple futures | Worker with shutdown signal |

### Channel Usage

| Channel | Type | Use Case |
|---------|------|----------|
| `mpsc::channel(bound)` | Multi-producer, single-consumer | Workers receiving commands from handlers |
| `broadcast::channel(cap)` | Multi-producer, multi-consumer | Broadcasting events to multiple WebSocket connections |
| `watch::channel(val)` | Single-producer, multi-consumer | Shared state (e.g., system health status) |
| `oneshot::channel()` | Single-use, single-value | Request-response within the system |

### Backpressure

All channels are bounded. When a channel is full:
- `send().await` blocks the sender (backpressure)
- `try_send()` returns an error immediately (non-blocking)
- The caller decides: wait, drop the message, or return an error to the client

This prevents unbounded memory growth from message backlogs.

---

*Next: [rust-architecture.md](rust-architecture.md) · [module-structure.md](module-structure.md)*
