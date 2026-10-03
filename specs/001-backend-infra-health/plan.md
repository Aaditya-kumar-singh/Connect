# Implementation Plan: Backend Infrastructure & Health Endpoints

**Branch**: `001-backend-infra-health` | **Date**: 2026-10-03 | **Spec**: [specs/001-backend-infra-health/spec.md](spec.md)

**Input**: Feature specification from `specs/001-backend-infra-health/spec.md` (Phase 1: Repository & Dev Infrastructure)

---

## Summary

Bootstrap the core Axum backend server skeleton in Rust with clean layered architecture:
1. Entry point in `backend/src/main.rs` that loads environment configuration via `config.rs`.
2. Asynchronous connection pool initialization for PostgreSQL (`sqlx::PgPool`) and Redis (`redis::aio::ConnectionManager`).
3. Shared `AppState` containing pools and application configuration.
4. Liveness probe (`GET /health`) returning version and server uptime without dependency access.
5. Readiness probe (`GET /ready`) testing PostgreSQL (`SELECT 1`) and Redis (`PING`) with a strict 2s timeout.
6. Middleware pipeline: CORS, request tracing/spans with Request ID, request timeout (30s), and compression.
7. Clean graceful shutdown on `SIGINT` / `SIGTERM` signals with connection drainage.
8. Unit and integration tests in `backend/tests/` asserting endpoint responses and status codes.

---

## Technical Context

**Language/Version**: Rust 2021 edition (Stable $\ge$ 1.75)  
**Primary Dependencies**: Axum 0.7, Tokio 1.x, Tower / Tower-HTTP 0.5, SQLx 0.8 (PostgreSQL), Redis 0.25, Serde / Serde-JSON, Tracing / Tracing-Subscriber  
**Storage**: PostgreSQL 16 (connection pooling via SQLx), Redis 7 (Tokio connection manager)  
**Testing**: `cargo test` (unit tests + integration tests against Axum router with `tower::ServiceExt`)  
**Target Platform**: Windows / Linux container (cross-platform Tokio async runtime)  
**Project Type**: Modular Monolith Backend Web Service  
**Performance Goals**: `GET /health` $< 10\text{ms}$, `GET /ready` $< 50\text{ms}$ under normal load  
**Constraints**: Zero downtime during rolling deploy; strict 2s timeout on dependency health checks; fail fast on invalid configuration  
**Scale/Scope**: Foundational infrastructure enabling all future modules (Auth, Messaging, WebRTC, Media)

---

## Constitution Check

*GATE: Must pass before implementation.*

- [x] **Modular Monolith & Strict Layering**: Axum handlers strictly handle HTTP transport and delegate checks via `AppState` services.
- [x] **PostgreSQL Durability & Truth**: Ready probe actively verifies PostgreSQL reachability using SQLx.
- [x] **Ephemeral Redis Coordination**: Ready probe verifies Redis ping without treating cache data as persistent.
- [x] **Zero-Trust Security**: No secrets or credentials exposed in health payloads; environment variables securely managed via typed config.
- [x] **Test-First Quality**: Integration tests using Axum test client verify both 200 OK and degraded 503 response contracts.

---

## Project Structure

### Documentation (this feature)

```text
specs/001-backend-infra-health/
├── spec.md              # Feature specification
├── plan.md              # Implementation plan (this document)
└── tasks.md             # Actionable task list
```

### Source Code Layout

```text
backend/
├── Cargo.toml
├── src/
│   ├── main.rs                   # Server entry point, signal handler, router assembly
│   ├── app_state.rs              # Shared AppState struct (PgPool, Redis, Config)
│   ├── config.rs                 # Validated environment configuration
│   ├── domain/
│   │   ├── mod.rs
│   │   └── errors.rs             # AppError & Result types
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── database.rs           # PostgreSQL connection pool creation & health check
│   │   └── redis_client.rs       # Redis connection manager creation & ping check
│   └── routes/
│       ├── mod.rs                # Router definition assembling all sub-routes
│       └── health.rs             # GET /health & GET /ready handlers and payload structs
└── tests/
    └── health_check.rs           # Integration tests for /health and /ready endpoints
```

---

## Architecture & Design Decisions

### 1. Separation of Liveness vs. Readiness
- **/health (Liveness)**: Only checks that the Axum process is running and event loop is responsive. Never touches the database or Redis. Orchestrators use this to know if the process has crashed.
- **/ready (Readiness)**: Actively runs concurrent `SELECT 1` on PostgreSQL and `PING` on Redis with `tokio::time::timeout(Duration::from_secs(2), ...)`. Returns 503 if any dependency fails, preventing traffic routing during outages or cold starts.

### 2. Standard Middleware Stack
- `TraceLayer`: Structured tracing for request method, path, status, and latency.
- `CorsLayer`: Permissive in development, origin-restricted in production.
- `TimeoutLayer`: 30-second request timeout to prevent hanging connections.
- `CompressionLayer`: Gzip compression for JSON responses.

### 3. Graceful Shutdown Flow
- Listens for `tokio::signal::ctrl_c()` and Unix `SIGTERM` (where supported).
- Once signaled, Axum stops accepting new connections and gives active requests up to 10 seconds to finish before process termination.

---

## Dependencies & External Services

- **PostgreSQL 16**: Port 5432 (`postgres://postgres:postgres@localhost:5432/ybm_connect`)
- **Redis 7**: Port 6379 (`redis://localhost:6379`)
- Local development runs via `docker compose up -d postgres redis`.

---

## Verification & Test Strategy

1. **Unit Tests**:
   - Configuration parsing and validation edge cases (missing variables, invalid port numbers).
2. **Integration Tests (`backend/tests/health_check.rs`)**:
   - `test_liveness_endpoint_returns_200`: Verify `GET /health` returns 200 with `{ "status": "ok" }`.
   - `test_readiness_endpoint_with_mock_or_live_db`: Verify `GET /ready` structure and status code.
3. **Manual CLI Verification**:
   - `cargo run` starts the server on configured port.
   - `curl -i http://localhost:8080/health` returns `200 OK`.
   - `curl -i http://localhost:8080/ready` returns `200 OK` (when containers are running) or `503 Service Unavailable` (when containers are down).
