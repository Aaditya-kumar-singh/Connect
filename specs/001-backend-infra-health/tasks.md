# Tasks: Backend Infrastructure & Health Endpoints

**Input**: Design documents from `specs/001-backend-infra-health/` (`spec.md`, `plan.md`)  
**Prerequisites**: `spec.md` (completed), `plan.md` (completed)  
**Tests**: Automated integration tests in `backend/tests/health_check.rs`

---

## Phase 1: Setup & Foundational Infrastructure

**Purpose**: Establish core types, error handling, and connection pool wrappers required by all endpoints.

- [x] T001 Verify and update `backend/src/domain/errors.rs` to support infrastructure, database, and Redis errors with Axum `IntoResponse` status code mappings.
- [x] T002 Enhance `backend/src/infrastructure/database.rs` with `create_pool(database_url)` and `check_health(&PgPool)` (executing `SELECT 1`).
- [x] T003 Enhance `backend/src/infrastructure/redis_client.rs` with `create_client(redis_url)` and `check_health(&redis::Client)` (executing `PING`).
- [x] T004 Update `backend/src/app_state.rs` to provide shared `AppState` containing `sqlx::PgPool`, `redis::Client`, and `Arc<Config>`.

**Checkpoint**: Foundation ready — domain errors, state, and backing clients available.

---

## Phase 2: User Story 1 - Liveness Probe (`GET /health`) (Priority: P1)

**Goal**: Expose a zero-dependency `/health` endpoint returning service status, version, and server timestamp.

- [x] T005 [P] [US1] Create integration test `test_health_liveness_returns_200` in `backend/tests/health_check.rs`.
- [x] T006 [US1] Implement `HealthResponse` struct and `health_check` handler in `backend/src/routes/health.rs`.
- [x] T007 [US1] Assemble `routes/mod.rs` routing module exposing `create_router(state: AppState) -> Router`.

**Checkpoint**: User Story 1 fully functional and testable independently.

---

## Phase 3: User Story 2 - Readiness Probe (`GET /ready`) (Priority: P1)

**Goal**: Expose a `/ready` endpoint that actively validates PostgreSQL and Redis availability with a 2-second timeout, returning 200 when ready or 503 when degraded.

- [x] T008 [P] [US2] Create integration tests for healthy and degraded readiness in `backend/tests/health_check.rs`. The healthy test is ignored unless `TEST_DATABASE_URL` and `TEST_REDIS_URL` are provided.
- [x] T009 [US2] Implement `ReadinessResponse`, `ComponentHealth` structs, and `readiness_check` handler in `backend/src/routes/health.rs` with `tokio::time::timeout`.

**Checkpoint**: User Story 2 fully functional and testable independently.

---

## Phase 4: User Story 3 - Server Lifecycle, Middleware & Graceful Shutdown (Priority: P2)

**Goal**: Provide production server startup, configuration validation, standard middleware, and graceful shutdown handling.

- [x] T010 [US3] Implement `backend/src/main.rs` with structured tracing subscriber, configuration validation, dependency-independent startup, middleware pipeline (CORS, request ID, TraceLayer, TimeoutLayer, compression), and SIGINT/SIGTERM graceful shutdown with a 10-second hard deadline.

**Checkpoint**: Full server binary runnable and verified with graceful shutdown.

---

## Phase 5: Verification & Polish

**Purpose**: Ensure compiler checks, formatting, and automated tests pass cleanly.

- [x] T011 Run `cargo test` in `backend/` to verify all unit and integration tests pass.
- [x] T012 Run `cargo clippy -- -D warnings` to enforce zero-warning code quality.
- [x] T013 Update agent context files. `AGENTS.md` and `GEMINI.md` now contain the mandatory Spec Kit + Ponytail AI development policy.
