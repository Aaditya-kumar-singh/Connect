# Feature Specification: Observability & Production Metrics

**Feature Branch**: `016-observability`
**Created**: 2026-10-04
**Status**: IMPLEMENTED
**Input**: Phase 16 of `docs/22-product/roadmap.md`, NFR-021 through NFR-024, and `docs/15-observability/logging.md`.

---

## User Scenarios & Testing

### User Story 1 - Prometheus Metrics Endpoint (P1)

As an operator, I need a Prometheus-compatible `GET /metrics` endpoint so request volume, latency, errors, WebSocket activity, and database pool usage can be scraped without changing application behavior.

**Acceptance Scenarios**
1. `GET /metrics` returns HTTP 200 with Prometheus text format.
2. The endpoint exposes HTTP request counters and latency histograms.
3. The endpoint exposes active WebSocket connections and total WebSocket connections.
4. The endpoint exposes PostgreSQL pool active/idle/max gauges.
5. Metrics collection never blocks application requests on external I/O.

### User Story 2 - HTTP Request Instrumentation (P1)

As an operator, I need every HTTP request measured by method, normalized path, and status so error rate and latency can be calculated.

**Acceptance Scenarios**
1. Every completed HTTP request increments `http_requests_total`.
2. Every completed HTTP request records `http_request_duration_seconds`.
3. Status code is recorded after the response is produced.
4. High-cardinality raw URLs, query strings, user IDs, tokens, and request bodies are never metric labels.

### User Story 3 - WebSocket Instrumentation (P1)

As an operator, I need connection and event counters to understand real-time load.

**Acceptance Scenarios**
1. Active connection gauge increments after successful registration and decrements during cleanup.
2. Total connection counter increments for each accepted WebSocket connection.
3. Received event counter is partitioned by event type.
4. Unknown or attacker-controlled event types do not create unbounded metric series.

### User Story 4 - Structured Request Context (P2)

As an operator, I need request IDs visible in structured logs so an HTTP request can be correlated across middleware and application logs.

**Acceptance Scenarios**
1. Request ID from `x-request-id` is available to the request tracing span.
2. Generated request IDs are propagated in the response.
3. Sensitive request data is not logged.

## Functional Requirements

- **FR-001**: Backend MUST expose `GET /metrics` in Prometheus text exposition format.
- **FR-002**: Backend MUST record HTTP request count and latency with bounded labels: method, normalized path, status.
- **FR-003**: Backend MUST record active and total WebSocket connections.
- **FR-004**: Backend MUST record WebSocket events using a bounded allowlist of event-type labels.
- **FR-005**: Backend MUST expose PostgreSQL pool active, idle, and max connection gauges.
- **FR-006**: Metrics MUST be held in process memory and MUST NOT depend on PostgreSQL, Redis, or object storage.
- **FR-007**: Metrics MUST NOT contain credentials, tokens, message bodies, email addresses, user IDs, or raw query strings.
- **FR-008**: HTTP tracing MUST include the request ID field when available.
- **FR-009**: Existing `/health` and `/ready` behavior MUST remain compatible.
- **FR-010**: Existing Rust behavior MUST remain unchanged apart from additive instrumentation.

## Non-Goals

- Grafana dashboards or alert delivery infrastructure.
- OpenTelemetry exporter deployment.
- Distributed trace storage.
- Business-level metrics for every future feature.
- Database query instrumentation for every SQL statement.

## Success Criteria

- `GET /metrics` is scrapeable and contains the required core metrics.
- `cargo test --lib`, integration tests, clippy, and formatting pass.
- Metrics use bounded labels and introduce no user-controlled cardinality.
- Existing health, authentication, messaging, WebSocket, and call behavior remains intact.
