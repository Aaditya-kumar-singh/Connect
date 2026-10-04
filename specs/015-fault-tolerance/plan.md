# Implementation Plan: Fault Tolerance & Reliability (Erlang/OTP Supervision Layer)

**Branch**: `015-fault-tolerance` | **Date**: 2026-10-04 | **Spec**: [specs/015-fault-tolerance/spec.md](spec.md)

**Input**: Feature specification from `specs/015-fault-tolerance/spec.md` (Phase 15: Fault Tolerance & Reliability)

---

## Summary

Introduce an Erlang/OTP reliability sidecar alongside the existing Rust/Axum modular monolith. The Erlang/OTP application provides:

1. An OTP supervision tree managing dependency monitors, circuit breakers, and a recovery coordinator.
2. Dependency health monitors that periodically probe PostgreSQL, Redis, and R2/MinIO.
3. Circuit breaker `gen_server` processes per dependency implementing CLOSED/OPEN/HALF_OPEN state machines.
4. A recovery coordinator that detects when degraded dependencies return to healthy state.
5. A health aggregation endpoint exposing combined system reliability status.
6. Redis Pub/Sub-based communication between Rust and Erlang (no new dependencies).
7. Docker Compose development service for the Erlang reliability node.
8. Graceful shutdown coordination between Rust and Erlang.
9. Erlang-side tests for supervision, circuit breakers, and dependency monitoring.
10. Minimal Rust-side changes: publish health/failure events on Redis Pub/Sub channels.

---

## Technical Context

**Language/Version (Erlang)**: Erlang/OTP 27, rebar3 build tool
**Language/Version (Rust)**: Rust 2021 edition (Stable ≥ 1.75), unchanged
**Communication**: Redis 7 Pub/Sub (channels: `ybm:reliability:events`, `ybm:reliability:status`)
**Testing (Erlang)**: rebar3 eunit + rebar3 ct (Common Test)
**Testing (Rust)**: cargo test (minimal additions for event publishing)
**Infrastructure**: Docker Compose (existing postgres, redis, minio + new reliability service)
**Constraints**: Erlang does not access PostgreSQL application data. Erlang does not serve client-facing APIs. Rust remains the primary backend runtime.

---

## Constitution Check

*GATE: Must pass before implementation.*

- [x] **Modular Monolith & Strict Layering**: Rust backend remains unchanged as the application monolith. Erlang/OTP is a separate sidecar with a single explicit communication boundary (Redis Pub/Sub). This satisfies the extraction criteria: independent failure domain, different runtime requirement.
- [x] **PostgreSQL Durability & Truth**: PostgreSQL remains the authoritative data store. Erlang does not write application data to PostgreSQL. Erlang only probes connectivity.
- [x] **Ephemeral Redis Coordination**: Erlang uses Redis Pub/Sub for event exchange, consistent with existing Redis usage patterns. No durable state in Redis.
- [x] **Zero-Trust Security**: No secrets, user tokens, or PII pass through the reliability channel. Erlang probes dependencies using its own minimal credentials.
- [x] **Test-First Quality**: Erlang eunit/CT tests verify supervision, circuit breakers, and monitoring. Rust tests verify event publishing.

---

## Project Structure

### Documentation (this feature)

```text
specs/015-fault-tolerance/
├── spec.md              # Feature specification
├── plan.md              # Implementation plan (this document)
└── tasks.md             # Actionable task list
```

### Source Code Layout (new Erlang application)

```text
reliability/
├── Dockerfile
├── rebar.config
├── rebar.lock
├── src/
│   ├── ybm_reliability.app.src
│   ├── ybm_reliability_app.erl        # OTP application behaviour
│   ├── ybm_reliability_sup.erl        # Root supervisor
│   ├── ybm_health_sup.erl             # Health aggregator supervisor
│   ├── ybm_health_aggregator.erl      # Combined health gen_server
│   ├── ybm_health_httpd.erl           # Inets /health callback
│   ├── ybm_dependency_sup.erl         # Dependency monitor supervisor
│   ├── ybm_pg_monitor.erl             # PostgreSQL health monitor
│   ├── ybm_redis_monitor.erl          # Redis health monitor
│   ├── ybm_storage_monitor.erl        # R2/MinIO health monitor
│   ├── ybm_circuit_sup.erl            # Circuit breaker supervisor
│   ├── ybm_circuit_breaker.erl        # Generic circuit breaker gen_server
│   ├── ybm_recovery_sup.erl           # Recovery supervisor
│   ├── ybm_recovery_coordinator.erl   # Recovery coordination gen_server
│   └── ybm_redis_pubsub.erl           # Redis Pub/Sub client
├── test/
│   ├── ybm_circuit_breaker_tests.erl  # Circuit breaker eunit tests
│   ├── ybm_supervisor_SUITE.erl       # Supervisor CT tests
│   └── ybm_monitor_tests.erl          # Monitor eunit tests
└── README.md
```

### Rust Changes (minimal)

```text
backend/src/
├── infrastructure/
│   └── reliability_events.rs          # Publish events on Redis Pub/Sub
└── (existing files: minimal changes to publish events)
```

---

## Architecture & Design Decisions

### 1. Communication via Redis Pub/Sub

**Decision**: Rust ↔ Erlang communication uses Redis Pub/Sub exclusively for advisory reliability events and status.

**Rationale**: Redis is already a project dependency. Adding Redis channels for reliability events requires zero new infrastructure. This follows the Ponytail principle of minimal new dependencies.

**Alternatives rejected**:
- gRPC: Adds protobuf dependency, code generation, and a new protocol. Not justified for the small message volume.
- Dedicated message broker (NATS/RabbitMQ): Adds operational complexity. Redis Pub/Sub is sufficient for advisory reliability events.
- Internal HTTP: Adds HTTP client/server boilerplate to both sides. Redis Pub/Sub is simpler.

### 2. Single Generic Circuit Breaker Module

**Decision**: One `ybm_circuit_breaker` gen_server module parameterized per dependency, rather than separate circuit breaker modules.

**Rationale**: All circuit breakers follow the same CLOSED/OPEN/HALF_OPEN state machine. Parameterizing by dependency name, thresholds, and cooldown avoids code duplication.

### 3. Erlang Probes Dependencies Independently

**Decision**: Erlang dependency monitors probe PostgreSQL, Redis, and R2 directly using minimal connectivity checks, rather than relying on Rust to report all health.

**Rationale**: If Rust itself crashes or becomes unresponsive, Erlang can still detect dependency health. This provides independent failure detection.

### 4. Erlang is Advisory, Not Blocking

**Decision**: Erlang publishes status events but does not gate Rust API operations. Rust continues to serve requests regardless of Erlang circuit state.

**Rationale**: Making Erlang a hard dependency would reduce overall reliability. Erlang provides visibility and coordination signals, but the Rust backend must remain functional even if Erlang is unavailable.

---

## Dependencies & External Services

- **Redis 7**: Port 6379 (existing, shared with Rust backend)
- **PostgreSQL 16**: Port 5432 (probe target only; Erlang uses `epgsql` or a raw TCP connect check)
- **MinIO/R2**: Port 9000 (probe target only; Erlang uses HTTP HEAD check)
- **Docker**: Build and run the Erlang container

### Erlang Dependencies (rebar3)

| Dependency | Purpose | Justification |
|-----------|---------|---------------|
| `eredis`  | Redis client for Pub/Sub | OTP stdlib has no Redis client. `eredis` is minimal and well-maintained. |

No other external Erlang dependencies are required. Dependency monitors use raw TCP/HTTP for probes without additional libraries. If `epgsql` is needed for PostgreSQL probes, it will be evaluated during implementation (a raw TCP connect to port 5432 may suffice).

---

## Verification & Test Strategy

### Erlang Tests

1. **Unit Tests (eunit)**:
   - Circuit breaker state transitions: CLOSED→OPEN, OPEN→HALF_OPEN, HALF_OPEN→CLOSED, HALF_OPEN→OPEN
   - Circuit breaker threshold counting
   - Health aggregation logic

2. **Common Test (ct)**:
   - Supervisor start/stop lifecycle
   - Child crash → restart
   - Max restart intensity → supervisor escalation
   - Sibling isolation: crash one child, verify others alive
   - Full application start/stop

3. **Integration Tests**:
   - Redis Pub/Sub: publish event, receive event
   - Dependency probe: connect to live PostgreSQL/Redis (when Docker containers running)

### Rust Tests

4. **Unit Tests**:
   - Event publishing: verify correct JSON structure on Redis channel

5. **Integration Tests**:
   - Verify Rust publishes events that are receivable on Redis Pub/Sub

### Manual Verification

6. **Docker Compose**:
   - `docker compose up` starts reliability service alongside existing services
   - Reliability service health endpoint returns healthy
   - Stop PostgreSQL → circuit opens → restart PostgreSQL → circuit closes
