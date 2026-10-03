# Local Development — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-DEVOPS-001`                           |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-DEVOPS-002 through 009                 |

---

## 1. Prerequisites

### Required Software

| Software | Version | Purpose | Windows Install |
|----------|---------|---------|-----------------|
| **Rust** | Latest stable (≥ 1.75) | Backend compilation | `winget install Rustlang.Rust.MSVC` or [rustup.rs](https://rustup.rs) |
| **Node.js** | LTS (≥ 20) | Frontend build/dev | `winget install OpenJS.NodeJS.LTS` |
| **Docker Desktop** | Latest | PostgreSQL, Redis containers | `winget install Docker.DockerDesktop` |
| **Git** | Latest | Version control | `winget install Git.Git` |
| **just** | Latest | Command runner (cross-platform Make alternative) | `cargo install just` |

### Optional but Recommended

| Software | Purpose | Install |
|----------|---------|---------|
| **sqlx-cli** | Database migrations | `cargo install sqlx-cli --features postgres` |
| **cargo-watch** | Auto-reload on code changes | `cargo install cargo-watch` |
| **cargo-nextest** | Faster test runner | `cargo install cargo-nextest` |

### Windows-Specific Notes
- **Docker Desktop** requires WSL 2 or Hyper-V enabled.
- Use **PowerShell 7+** or **Windows Terminal** for best experience.
- Git should be configured with `core.autocrlf = true` on Windows.
- Rust on Windows uses the MSVC toolchain by default; this is correct.

## 2. Quick Start (< 15 Minutes)

```powershell
# 1. Clone the repository
git clone https://github.com/your-org/ybm-connect.git
cd ybm-connect

# 2. Copy environment files
copy .env.example .env

# 3. Start infrastructure (PostgreSQL + Redis)
docker compose up -d

# 4. Run database migrations
just db-migrate

# 5. Seed development data (optional)
just db-seed

# 6. Start the backend
just dev-backend

# 7. (In a new terminal) Start the web frontend
just dev-web

# 8. (In a new terminal) Start the mobile dev server (if developing mobile)
just dev-mobile
```

Or, use the single command:

```powershell
just dev
```

This runs steps 3-7 concurrently.

## 3. Environment Configuration

### `.env.example`

```env
# ============================================================
# YBM Connect — Local Development Environment
# ============================================================
# Copy this file to .env and customize as needed.
# NEVER commit .env to Git.
# ============================================================

# ---- Server ----
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
RUST_LOG=ybm_connect=debug,tower_http=debug,sqlx=warn

# ---- Database ----
DATABASE_URL=postgres://ybm:ybm_dev_password@localhost:5432/ybm_connect
DATABASE_MAX_CONNECTIONS=10
DATABASE_MIN_CONNECTIONS=2

# ---- Redis ----
REDIS_URL=redis://localhost:6379
REDIS_MAX_CONNECTIONS=10

# ---- Authentication ----
ACCESS_TOKEN_SECRET=dev-access-token-secret-change-in-production
REFRESH_TOKEN_SECRET=dev-refresh-token-secret-change-in-production
ACCESS_TOKEN_EXPIRY_SECONDS=900
REFRESH_TOKEN_EXPIRY_DAYS=30
ARGON2_MEMORY_KB=65536
ARGON2_ITERATIONS=3
ARGON2_PARALLELISM=4

# ---- Email ----
EMAIL_PROVIDER=console
# In development, OTPs are logged to console instead of sending email.
# For real email: EMAIL_PROVIDER=resend, RESEND_API_KEY=re_xxxxx
EMAIL_FROM=noreply@localhost

# ---- Media / R2 ----
R2_ACCOUNT_ID=dev-account-id
R2_ACCESS_KEY_ID=dev-access-key
R2_SECRET_ACCESS_KEY=dev-secret-key
R2_BUCKET_NAME=ybm-connect-dev
R2_ENDPOINT=http://localhost:9000
# For local dev, we use MinIO as an S3-compatible replacement for R2.

# ---- TURN ----
TURN_SERVER_URL=turn:localhost:3478
TURN_SECRET=dev-turn-secret
TURN_REALM=ybm-connect.local

# ---- Push Notifications ----
PUSH_PROVIDER=console
# In development, push notifications are logged to console.
# FCM_CREDENTIALS_PATH=./credentials/firebase.json
# VAPID_PRIVATE_KEY=...
# VAPID_PUBLIC_KEY=...

# ---- Frontend ----
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_WS_URL=ws://localhost:8080/ws/connect

# ---- CORS ----
CORS_ALLOWED_ORIGINS=http://localhost:3000,http://localhost:19006
```

## 4. Docker Compose

### `docker-compose.yml`

```yaml
version: "3.9"

services:
  postgres:
    image: postgres:16-alpine
    container_name: ybm-postgres
    environment:
      POSTGRES_USER: ybm
      POSTGRES_PASSWORD: ybm_dev_password
      POSTGRES_DB: ybm_connect
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ybm -d ybm_connect"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    container_name: ybm-redis
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5

  minio:
    image: minio/minio:latest
    container_name: ybm-minio
    command: server /data --console-address ":9001"
    environment:
      MINIO_ROOT_USER: dev-access-key
      MINIO_ROOT_PASSWORD: dev-secret-key
    ports:
      - "9000:9000"
      - "9001:9001"
    volumes:
      - minio_data:/data

volumes:
  pgdata:
  minio_data:
```

## 5. Justfile (Cross-Platform Command Runner)

```makefile
# YBM Connect — Development Commands

# Start all development services
dev:
    docker compose up -d
    @echo "Waiting for services..."
    just _wait-for-postgres
    just _wait-for-redis
    just db-migrate
    @echo "Starting backend and frontend..."
    just dev-backend &
    just dev-web

# Start only the backend (with auto-reload)
dev-backend:
    cargo watch -x 'run' -w src/

# Start only the web frontend
dev-web:
    cd frontend/web && npm run dev

# Start mobile dev server
dev-mobile:
    cd frontend/mobile && npx expo start

# Database migrations
db-migrate:
    sqlx migrate run --source backend/migrations

db-migrate-down:
    sqlx migrate revert --source backend/migrations

db-reset:
    sqlx database drop -y
    sqlx database create
    sqlx migrate run --source backend/migrations
    just db-seed

db-seed:
    cargo run --bin seed

# Testing
test:
    cargo test --lib
    cargo test --test integration

test-unit:
    cargo test --lib

test-integration:
    cargo test --test integration

test-watch:
    cargo watch -x 'test --lib'

# Linting
lint:
    cargo clippy -- -D warnings
    cd frontend/web && npx eslint .

fmt:
    cargo fmt
    cd frontend/web && npx prettier --write .

# Build
build-backend:
    cargo build --release

build-web:
    cd frontend/web && npm run build

# Docker helper
_wait-for-postgres:
    @powershell -Command "while (!(Test-NetConnection -ComputerName localhost -Port 5432 -InformationLevel Quiet -ErrorAction SilentlyContinue)) { Start-Sleep 1 }"

_wait-for-redis:
    @powershell -Command "while (!(Test-NetConnection -ComputerName localhost -Port 6379 -InformationLevel Quiet -ErrorAction SilentlyContinue)) { Start-Sleep 1 }"
```

## 6. Repository Structure

```
ybm-connect/
├── .env.example                    # Environment template
├── .gitignore                      # Git ignore rules
├── docker-compose.yml              # Local infrastructure
├── justfile                        # Development commands
├── README.md                       # Quick start guide
│
├── backend/                        # Rust backend
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── migrations/                 # SQL migrations
│   │   ├── 001_create_users.up.sql
│   │   ├── 001_create_users.down.sql
│   │   └── ...
│   ├── src/
│   │   ├── main.rs                 # Entry point
│   │   ├── config.rs               # Configuration loading
│   │   ├── app_state.rs            # Shared application state
│   │   ├── domain/                 # Domain types
│   │   │   ├── mod.rs
│   │   │   ├── user.rs
│   │   │   ├── message.rs
│   │   │   ├── conversation.rs
│   │   │   └── ...
│   │   ├── auth/                   # Authentication module
│   │   │   ├── mod.rs
│   │   │   ├── handlers.rs
│   │   │   ├── service.rs
│   │   │   ├── repository.rs
│   │   │   └── errors.rs
│   │   ├── messaging/              # Messaging module
│   │   ├── groups/                 # Groups module
│   │   ├── media/                  # Media module
│   │   ├── calls/                  # Call signaling module
│   │   ├── presence/               # Presence module
│   │   ├── notifications/          # Push notifications
│   │   ├── websocket/              # WebSocket management
│   │   ├── workers/                # Background workers
│   │   │   ├── mod.rs
│   │   │   ├── supervisor.rs
│   │   │   └── ...
│   │   ├── middleware/             # Axum middleware
│   │   │   ├── auth.rs
│   │   │   ├── rate_limit.rs
│   │   │   ├── request_id.rs
│   │   │   └── logging.rs
│   │   └── infrastructure/        # External adapters
│   │       ├── database.rs
│   │       ├── redis.rs
│   │       ├── r2.rs
│   │       ├── email.rs
│   │       └── push.rs
│   └── tests/                     # Integration tests
│       ├── common/mod.rs
│       ├── auth_test.rs
│       ├── messaging_test.rs
│       └── ...
│
├── frontend/
│   ├── web/                       # Next.js web app
│   │   ├── package.json
│   │   ├── next.config.js
│   │   ├── src/
│   │   │   ├── app/               # Next.js App Router
│   │   │   ├── components/
│   │   │   ├── hooks/
│   │   │   ├── lib/               # API clients, WebSocket
│   │   │   ├── stores/            # State management
│   │   │   └── types/
│   │   └── public/
│   │
│   └── mobile/                    # Expo React Native app
│       ├── package.json
│       ├── app.json
│       ├── src/
│       │   ├── screens/
│       │   ├── components/
│       │   ├── hooks/
│       │   ├── lib/
│       │   ├── stores/
│       │   └── types/
│       └── assets/
│
├── docs/                          # This documentation tree
│   ├── 00-overview/
│   ├── 01-requirements/
│   └── ...
│
├── scripts/                       # Utility scripts
│   ├── wait-for-deps.sh
│   └── generate-turn-creds.sh
│
└── .github/                       # CI/CD
    └── workflows/
        ├── ci.yml
        └── deploy.yml
```

## 7. Common Development Tasks

| Task | Command |
|------|---------|
| Start everything | `just dev` |
| Backend only (with hot reload) | `just dev-backend` |
| Web frontend only | `just dev-web` |
| Run all tests | `just test` |
| Run only unit tests | `just test-unit` |
| Run only integration tests | `just test-integration` |
| Create a migration | `sqlx migrate add <name> --source backend/migrations` |
| Apply migrations | `just db-migrate` |
| Revert last migration | `just db-migrate-down` |
| Reset database | `just db-reset` |
| Lint code | `just lint` |
| Format code | `just fmt` |
| Build release binary | `just build-backend` |

---

*Next: [environment-management.md](environment-management.md) · [docker.md](docker.md)*
