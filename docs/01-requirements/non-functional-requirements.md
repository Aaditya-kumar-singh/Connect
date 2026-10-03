# Non-Functional Requirements — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-REQ-003`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-REQ-001, DOC-REQ-002, DOC-PERF-001    |

---

## Performance

### NFR-001: Message Delivery Latency
**Requirement:** p95 message delivery latency (sender sends → recipient receives) MUST be < 500ms within the same deployment region under normal load.
**Measurement:** Timestamp comparison at sender and recipient (clock skew compensated via server timestamp).
**Related:** BR-001, MSG-001

### NFR-002: REST API Latency
**Requirement:** p95 REST API response time MUST be < 200ms for non-media endpoints.
**Measurement:** Server-side request duration histogram.
**Related:** BR-001

### NFR-003: WebSocket Connection Establishment
**Requirement:** WebSocket upgrade + authentication MUST complete in < 1 second (p95).
**Measurement:** Client-side timing from initiation to first usable message.

### NFR-004: Database Query Latency
**Requirement:** p95 database query latency MUST be < 50ms for indexed queries.
**Measurement:** Query timing middleware in the repository layer.
**Related:** DB-001

### NFR-005: Concurrent Connections
**Requirement:** A single backend instance MUST support at least 1,000 concurrent WebSocket connections.
**Measurement:** Load test with simulated clients.
**Related:** PERF-001

### NFR-006: Message Throughput
**Requirement:** A single backend instance MUST process at least 500 messages per second.
**Measurement:** Load test with concurrent senders.
**Related:** PERF-001

### NFR-007: Call Setup Time
**Requirement:** Time from call initiation to media flowing MUST be < 5 seconds (p95) when STUN succeeds, < 8 seconds when TURN fallback is needed.
**Measurement:** Client-side timing from offer to ontrack event.
**Related:** CALL-001

---

## Reliability

### NFR-008: Message Durability
**Requirement:** Once a message is acknowledged by the server, it MUST NOT be lost, even if the server crashes immediately after acknowledgement.
**Design:** The ACK is sent only after the PostgreSQL transaction commits.
**Related:** MSG-001, MSG-002

### NFR-009: Availability Target
**Requirement:** 99.5% uptime for the messaging service (allows ~3.6 hours downtime per month).
**Measurement:** Health check endpoint monitoring.
**Note:** This target is for the portfolio deployment. Production would target 99.9%+.

### NFR-010: Graceful Degradation
**Requirement:** If Redis is unavailable, the system MUST continue to function for message send/receive (without presence, typing indicators, or cross-instance routing). If PostgreSQL is unavailable, the system MUST return appropriate error responses without crashing.
**Related:** FT-001

### NFR-011: Recovery Time
**Requirement:** The system MUST recover from a single component failure (Redis restart, backend restart) within 60 seconds without manual intervention.
**Related:** FT-001

---

## Scalability

### NFR-012: Horizontal Scalability
**Requirement:** The architecture MUST allow adding more backend instances to handle increased load. Adding an instance MUST NOT require code changes.
**Design:** Stateless backend (session state in Redis/PostgreSQL), Redis pub/sub for cross-instance messaging.
**Related:** ADR-011

### NFR-013: Database Scalability
**Requirement:** The database schema MUST support read replicas without schema changes. Partitioning strategy for messages table MUST be documented (even if not implemented in V1).
**Related:** DB-001

### NFR-014: Connection Scalability
**Requirement:** The system MUST support scaling WebSocket connections by adding backend instances behind a load balancer with sticky sessions or Redis-based routing.
**Related:** NFR-005

---

## Security

### NFR-015: Password Storage
**Requirement:** Passwords MUST be hashed with Argon2id. Plaintext passwords MUST NOT be stored, logged, or transmitted after initial receipt.
**Parameters:** Argon2id with memory=64MB, iterations=3, parallelism=4 (tuned per deployment hardware).
**Related:** AUTH-001, SEC-001

### NFR-016: Transport Security
**Requirement:** All client-server communication MUST use TLS 1.2+ (HTTPS/WSS). Internal service communication MUST use TLS or private networking.
**Related:** SEC-001

### NFR-017: Token Security
**Requirement:** Access tokens MUST expire within 15 minutes. Refresh tokens MUST expire within 30 days. Token rotation MUST be implemented.
**Related:** AUTH-004, SEC-001

### NFR-018: Rate Limiting
**Requirement:** All public endpoints MUST be rate-limited. Rate limits are defined per-endpoint in the API documentation.
**Related:** SEC-001

### NFR-019: Input Validation
**Requirement:** All user input MUST be validated server-side. Client-side validation is for UX only and MUST NOT be relied upon for security.
**Related:** SEC-001

### NFR-020: Authorization
**Requirement:** Every API endpoint and WebSocket event MUST verify that the authenticated user is authorized to perform the requested action on the requested resource.
**Related:** SEC-001

---

## Observability

### NFR-021: Structured Logging
**Requirement:** All log entries MUST be structured (JSON format) with at minimum: timestamp, level, service, request_id. Sensitive data MUST NOT be logged.
**Related:** OBS-001

### NFR-022: Health Checks
**Requirement:** The backend MUST expose `/health` (liveness) and `/ready` (readiness) endpoints. Readiness checks PostgreSQL and Redis connectivity.
**Related:** OBS-001

### NFR-023: Metrics
**Requirement:** The backend MUST expose Prometheus-compatible metrics at `/metrics` including: request count, latency histogram, active WebSocket connections, error rate, database connection pool usage.
**Related:** OBS-001

### NFR-024: Request Tracing
**Requirement:** Every request MUST be assigned a unique `request_id`. This ID MUST be included in all log entries, error responses, and propagated to downstream calls.
**Related:** OBS-001

---

## Maintainability

### NFR-025: Code Organization
**Requirement:** Backend code MUST follow a layered architecture: Handler → Service → Repository → Database. Business logic MUST NOT exist in handlers or repository layers.
**Related:** ADR-011

### NFR-026: Test Coverage
**Requirement:** All business logic functions MUST have unit tests. All API endpoints MUST have integration tests. WebSocket flows MUST have integration tests.
**Target:** 80% code coverage for business logic modules.

### NFR-027: Documentation Currency
**Requirement:** Architecture documentation MUST be updated when architecture changes. API documentation MUST be updated when APIs change. Stale documentation is a bug.

---

## Compatibility

### NFR-028: API Versioning
**Requirement:** REST APIs MUST be versioned via URL path (`/api/v1/...`). Breaking changes MUST increment the version. The previous version MUST be supported for at least one release cycle.
**Related:** API-001

### NFR-029: Browser Support
**Requirement:** The web application MUST support the last 2 major versions of Chrome, Firefox, Safari, and Edge.

### NFR-030: Android Support
**Requirement:** The Android application MUST support Android 10 (API level 29) and above.

---

*Next: [user-stories.md](user-stories.md) · [use-cases.md](use-cases.md)*
