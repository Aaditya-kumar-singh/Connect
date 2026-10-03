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
