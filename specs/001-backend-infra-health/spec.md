# Feature Specification: Backend Infrastructure & Health Endpoints

**Feature Branch**: `001-backend-infra-health`  
**Created**: 2026-10-03  
**Status**: Draft  
**Input**: Phase 1 of Implementation Roadmap (`docs/22-product/roadmap.md`): Backend project skeleton, Axum HTTP server, configuration loader, database/cache connection pools, and health/readiness endpoints.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Service Liveness Probe (Priority: P1)

As an orchestration system (Docker, Kubernetes) or operational engineer, I need a lightweight `/health` endpoint that confirms the web server process is alive, listening, and accepting HTTP requests, so that failing containers can be detected and restarted automatically.

**Why this priority**: Without a basic liveness endpoint, automated orchestrators cannot monitor container process health.

**Independent Test**:
Can be fully tested by sending an HTTP `GET /health` request to the running server without needing active database or Redis connections. Must return `200 OK` with JSON `{ "status": "ok", "version": "0.1.0", "timestamp": "..." }`.

**Acceptance Scenarios**:
1. **Given** the Axum backend server is running, **When** a client sends `GET /health`, **Then** the server responds with HTTP `200 OK` and a JSON body containing service status and version within 50ms.
2. **Given** the server is processing high traffic, **When** `GET /health` is queried, **Then** it responds promptly without acquiring database locks or performing blocking I/O.

---

### User Story 2 - Comprehensive Dependency Readiness Probe (Priority: P1)

As a load balancer or deployment controller, I need a `/ready` endpoint that verifies connectivity to critical backing infrastructure (PostgreSQL database and Redis cache), so that incoming traffic is routed only when all dependencies are healthy and operational.

**Why this priority**: Prevents user-facing requests from failing when the database or cache connections are degraded, deadlocked, or during cold starts before migrations complete.

**Independent Test**:
Can be tested by simulating healthy and degraded dependency states:
- When both PostgreSQL and Redis are accessible, returns `200 OK` with detailed component health.
- When either PostgreSQL or Redis is unreachable, returns `503 Service Unavailable` with specific failure reasons for the degraded dependency.

**Acceptance Scenarios**:
1. **Given** both PostgreSQL and Redis connection pools are healthy and responsive, **When** a client sends `GET /ready`, **Then** the server responds with HTTP `200 OK` and JSON:
   ```json
   {
     "status": "ready",
     "database": { "status": "healthy", "latency_ms": 2 },
     "redis": { "status": "healthy", "latency_ms": 1 }
   }
   ```
2. **Given** PostgreSQL is stopped or unreachable, **When** a client sends `GET /ready`, **Then** the server responds with HTTP `503 Service Unavailable` indicating the database check failed, while still returning within timeout limits.
3. **Given** Redis is unreachable, **When** a client sends `GET /ready`, **Then** the server responds with HTTP `503 Service Unavailable` reporting Redis degradation.

---

### User Story 3 - Graceful Server Lifecycle & Structured Logging (Priority: P2)

As a DevOps engineer and backend developer, I need the server to initialize with validated environment configurations, structured JSON logging, standard middleware (CORS, request tracing, timeouts), and cleanly shut down on SIGINT/SIGTERM, so that in-flight requests finish without dropped connections.

**Why this priority**: Required for production stability, zero-downtime rolling deploys, and clear observability during development and debugging.

**Independent Test**:
Can be tested by passing invalid configuration (fails fast on startup with informative error) and by sending `SIGINT` (Ctrl+C) while a request is in flight to verify graceful connection drainage.

**Acceptance Scenarios**:
1. **Given** required environment variables (e.g. invalid port or missing DB URL) are malformed, **When** `cargo run` is executed, **Then** the application logs an actionable configuration error and exits with code 1.
2. **Given** the backend is handling requests, **When** a termination signal (`SIGINT` or `SIGTERM`) is received, **Then** the server stops accepting new connections, drains existing connections within a grace period (e.g., 10s), and flushes logs before exiting with code 0.

---

### Edge Cases

- **Slow backing service**: If PostgreSQL or Redis responds sluggishly, readiness check MUST enforce an aggressive timeout (e.g., 2000ms) rather than hanging indefinitely.
- **Connection pool exhaustion**: If all connection pool slots are occupied, the readiness probe should report pool saturation rather than deadlocking.
- **Rapid signal delivery**: Multiple rapid `SIGINT` signals should force immediate termination if graceful shutdown exceeds the shutdown timeout.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a `GET /health` endpoint that returns HTTP 200 with service version, status, and server timestamp without depending on external services.
- **FR-002**: System MUST provide a `GET /ready` endpoint that actively tests PostgreSQL connectivity (via `SELECT 1`) and Redis connectivity (via `PING`).
- **FR-003**: System MUST return HTTP 503 on `GET /ready` whenever any required backing service (PostgreSQL or Redis) is unreachable or fails to respond within 2000ms.
- **FR-004**: System MUST load and validate environment variables from `.env` or system environment using the existing `Config` struct.
- **FR-005**: System MUST initialize `sqlx::PgPool` and `redis::Client` connection managers and inject them into Axum's shared application state (`AppState`).
- **FR-006**: System MUST attach standard HTTP middleware: CORS (configurable allowed origins), request ID tracing with structured tracing spans, request timeout (30s), and gzip compression.
- **FR-007**: System MUST intercept `SIGINT` and `SIGTERM` signals and execute a graceful shutdown sequence.
- **FR-008**: System MUST provide automated integration tests asserting HTTP responses for `/health` and `/ready`.

### Key Entities

- **HealthResponse**: Representation of the service liveness state (fields: `status`, `version`, `timestamp`).
- **ReadinessResponse**: Representation of service readiness and subsystem health (fields: `status`, `components` with sub-entries for `database` and `redis`).
- **AppState**: Shared, cloneable Axum state containing database connection pool (`PgPool`), Redis connection client, and configuration settings.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `GET /health` returns HTTP 200 with JSON payload in `< 10ms` under normal conditions.
- **SC-002**: `GET /ready` returns HTTP 200 within `< 50ms` when dependencies are healthy, and fails fast with HTTP 503 within `< 2500ms` when a dependency is dead.
- **SC-003**: Server boots up in `< 1500ms` in a local development environment.
- **SC-004**: Automated test suite (`cargo test`) executes and passes 100% of unit and integration tests without flakiness.

---

## Assumptions

- PostgreSQL and Redis instances are running locally or via `docker compose up -d` using settings from `.env.example`.
- Authentication is not required for `/health` and `/ready` endpoints, as they are accessed by internal orchestrators and load balancers.
- Future application routes (Auth, Users, Messages) will mount directly onto this foundational Axum router and consume `AppState`.
