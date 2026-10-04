# Environment Variables Reference — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-REF-003`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |

---

## Backend Environment Variables

| Variable | Required | Default | Description | Sensitive |
|----------|----------|---------|-------------|-----------|
| `SERVER_HOST` | No | `127.0.0.1` | HTTP server bind address | No |
| `PORT` | No | — | Platform-provided HTTP server port; takes precedence over `SERVER_PORT` | No |
| `SERVER_PORT` | No | `8080` | Local/default HTTP server port | No |
| `RUST_LOG` | No | `ybm_connect=info` | Log level filter | No |
| `DATABASE_URL` | **Yes** | — | PostgreSQL connection string | **Yes** |
| `DATABASE_MAX_CONNECTIONS` | No | `10` | Max DB pool connections | No |
| `DATABASE_MIN_CONNECTIONS` | No | `2` | Min DB pool connections | No |
| `REDIS_URL` | **Yes** | — | Redis connection string | **Yes** |
| `REDIS_MAX_CONNECTIONS` | No | `10` | Max Redis pool connections | No |
| `ACCESS_TOKEN_SECRET` | **Yes** | — | HMAC secret for access tokens | **Yes** |
| `REFRESH_TOKEN_SECRET` | **Yes** | — | HMAC secret for refresh tokens | **Yes** |
| `ACCESS_TOKEN_EXPIRY_SECONDS` | No | `900` (15 min) | Access token lifetime | No |
| `REFRESH_TOKEN_EXPIRY_DAYS` | No | `30` | Refresh token lifetime | No |
| `ARGON2_MEMORY_KB` | No | `65536` (64MB) | Argon2id memory parameter | No |
| `ARGON2_ITERATIONS` | No | `3` | Argon2id time parameter | No |
| `ARGON2_PARALLELISM` | No | `4` | Argon2id parallelism | No |
| `EMAIL_PROVIDER` | No | `console` | Email provider: `console`, `resend`, `smtp` | No |
| `EMAIL_FROM` | No | `noreply@localhost` | Sender email address | No |
| `RESEND_API_KEY` | If using Resend | — | Resend API key | **Yes** |
| `SMTP_HOST` | If using SMTP | — | SMTP server host | No |
| `SMTP_PORT` | If using SMTP | `587` | SMTP server port | No |
| `SMTP_USERNAME` | If using SMTP | — | SMTP auth username | **Yes** |
| `SMTP_PASSWORD` | If using SMTP | — | SMTP auth password | **Yes** |
| `R2_ENABLED` | No | `true` | Enable S3-compatible media storage | No |
| `R2_ACCOUNT_ID` | If using R2 | — | Cloudflare account ID | No |
| `R2_ACCESS_KEY_ID` | If using R2 | — | R2 access key | **Yes** |
| `R2_SECRET_ACCESS_KEY` | If using R2 | — | R2 secret key | **Yes** |
| `R2_BUCKET_NAME` | If using R2 | — | R2 bucket name | No |
| `R2_ENDPOINT` | If using R2 | — | R2 endpoint URL | No |
| `TURN_SERVER_URL` | No | — | TURN server URL | No |
| `TURN_SECRET` | If using TURN | — | TURN shared secret | **Yes** |
| `TURN_REALM` | If using TURN | — | TURN realm | No |
| `PUSH_PROVIDER` | No | `console` | Push provider: `console`, `fcm`, `webpush` | No |
| `FCM_CREDENTIALS_PATH` | If using FCM | — | Path to Firebase credentials JSON | **Yes** |
| `VAPID_PRIVATE_KEY` | If using Web Push | — | VAPID private key | **Yes** |
| `VAPID_PUBLIC_KEY` | If using Web Push | — | VAPID public key for browser subscriptions | No |
| `VAPID_SUBJECT` | If using Web Push | — | VAPID contact URL (`mailto:` or `https://`) | No |
| `CORS_ALLOWED_ORIGINS` | No | `http://localhost:3000` | Comma-separated CORS origins | No |
| `MAX_UPLOAD_SIZE_MB` | No | `50` | Max upload file size in MB | No |
| `GRACEFUL_SHUTDOWN_TIMEOUT_SECONDS` | No | `30` | Max seconds for graceful shutdown | No |

## Erlang/OTP Reliability Service

| Variable | Required | Default | Description | Sensitive |
|----------|----------|---------|-------------|-----------|
| `REDIS_HOST` | No | `127.0.0.1` | Redis hostname for reliability Pub/Sub/probes | No |
| `REDIS_PORT` | No | `6379` | Redis port | No |
| `REDIS_PASSWORD` | No | empty | Redis authentication password | **Yes** |
| `PG_HOST` | No | `127.0.0.1` | PostgreSQL hostname for connectivity probe | No |
| `PG_PORT` | No | `5432` | PostgreSQL port | No |
| `R2_ENDPOINT` | No | `http://127.0.0.1:9000` | R2/MinIO endpoint for storage probe | No |
| `HEALTH_PORT` | No | `8081` | Reliability health endpoint port | No |
| `PROBE_INTERVAL_PG` | No | `15000` | PostgreSQL probe interval in milliseconds | No |
| `PROBE_INTERVAL_REDIS` | No | `10000` | Redis probe interval in milliseconds | No |
| `PROBE_INTERVAL_STORAGE` | No | `30000` | R2/MinIO probe interval in milliseconds | No |

## Frontend Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `NEXT_PUBLIC_API_URL` | **Yes** | — | Backend API URL |
| `NEXT_PUBLIC_WS_URL` | **Yes** | — | Backend WebSocket URL |
| `NEXT_PUBLIC_VAPID_PUBLIC_KEY` | If using Web Push | — | VAPID public key for push subscription |

## Ports

| Port | Service | Protocol | Notes |
|------|---------|----------|-------|
| `3000` | Next.js dev server | HTTP | Frontend development only |
| `8080` | Axum backend | HTTP/WS | API + WebSocket |
| `5432` | PostgreSQL | PostgreSQL | Database |
| `6379` | Redis | Redis | Cache + pub/sub |
| `9000` | MinIO (dev) | HTTP | S3-compatible local object storage |
| `9001` | MinIO Console (dev) | HTTP | MinIO web UI |
| `8081` | Erlang Reliability | HTTP | Reliability health endpoint |
| `3478` | coturn STUN | UDP/TCP | STUN NAT traversal |
| `3478` | coturn TURN | UDP/TCP | TURN media relay |
| `5349` | coturn TLS | TCP | TURN over TLS |
| `49152-65535` | coturn relay | UDP | TURN relay ports |

---

*Next: [configuration.md](configuration.md) · [services.md](services.md)*
