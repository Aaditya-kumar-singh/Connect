# Phase 15 — Fault Tolerance & Reliability (Erlang/OTP Supervision Layer)

| Field            | Value                                        |
|------------------|----------------------------------------------|
| **Spec ID**      | `SPEC-015`                                   |
| **Version**      | `1.0.0`                                      |
| **Status**       | `APPROVED`                                      |
| **Owner**        | Engineering Lead                             |
| **Created**      | 2026-10-04                                   |
| **Related Docs** | DOC-ARCH-001, DOC-FT-001, ADR-012, ADR-018  |

---

## Scope

Introduce an Erlang/OTP reliability runtime as a dedicated sidecar service alongside the existing Rust/Axum backend. The Erlang/OTP layer is responsible exclusively for fault-tolerance concerns: supervision, dependency health monitoring, circuit breakers, controlled retries, failure isolation, and recovery coordination.

### In Scope

- OTP supervision tree for reliability processes
- Dependency health monitoring (PostgreSQL, Redis, R2, push providers)
- Circuit breaker state management for external dependencies
- Retry coordination with backoff policies
- Graceful shutdown coordination between Rust and Erlang runtimes
- Rust ↔ Erlang communication boundary via Redis Pub/Sub (`events` and `status` channels only)
- Docker Compose development service for the Erlang node
- Recovery orchestration when degraded dependencies return
- Minimum observability signals (health, circuit state, restart counts)

### Out of Scope

- Rewriting any Rust application/business logic in Erlang
- Migrating WebSocket, REST API, or WebRTC signaling to Erlang
- Migrating PostgreSQL/Redis data access to Erlang
- Distributed Erlang clustering (single-node OTP only)
- Full observability platform (Phase 16)
- Kubernetes or service mesh infrastructure
- Additional message brokers (Kafka, RabbitMQ, NATS)
- Group call or media processing changes
- Any changes to the PostgreSQL schema
- Any changes to client-facing API contracts

---

## Architecture Decision

### Rust (unchanged responsibilities)

- REST APIs, authentication, authorization
- Messaging, conversations, users, groups
- Media upload/download, notification dispatch
- WebSocket application protocol, WebRTC signaling
- PostgreSQL access (SQLx), Redis access, R2 access
- Validation, business rules, API contracts
- All application-level error handling

### Erlang/OTP (new responsibilities)

- Root supervision tree managing reliability processes
- Dependency monitors: periodic health probes for PostgreSQL, Redis, R2
- Circuit breakers: per-dependency state machines (CLOSED → OPEN → HALF_OPEN)
- Retry coordination: backoff policies for recovery attempts
- Worker supervision: restart crashed reliability workers with intensity limits
- Recovery coordinator: detect when degraded dependencies recover
- Health aggregation: expose combined system health status

### Communication Boundary

Rust and Erlang communicate exclusively through **Redis Pub/Sub channels**:

- Channel `ybm:reliability:events` — Rust publishes health/failure events
- Channel `ybm:reliability:status` — Erlang publishes aggregated status
This reuses the existing Redis infrastructure without adding a new broker. The Erlang service may use a minimal Redis client library because OTP does not provide a Redis client; this dependency must remain limited to the reliability service. Redis is already a project dependency used for presence, typing, and message routing.

### Data Ownership

| Data                    | Owner       | Store      |
|-------------------------|-------------|------------|
| Users, messages, groups | Rust        | PostgreSQL |
| Sessions, presence      | Rust        | Redis      |
| Circuit breaker state   | Erlang/OTP  | In-process |
| Restart counts          | Erlang/OTP  | In-process |
| Dependency health       | Erlang/OTP  | In-process |

Erlang/OTP does NOT persist any application data. Its state is ephemeral and reconstructed on startup through health probes.

---

## Supervision Tree

```
ybm_reliability_sup (root, one_for_one)
│
├── ybm_health_sup (one_for_one)
│   └── ybm_health_aggregator (gen_server)
│
├── ybm_dependency_sup (one_for_one)
│   ├── ybm_pg_monitor (gen_server)
│   ├── ybm_redis_monitor (gen_server)
│   └── ybm_storage_monitor (gen_server)
│
├── ybm_circuit_sup (one_for_one)
│   ├── ybm_pg_circuit (gen_server)
│   ├── ybm_redis_circuit (gen_server)
│   └── ybm_storage_circuit (gen_server)
│
└── ybm_recovery_sup (one_for_one)
    └── ybm_recovery_coordinator (gen_server)
```

### Process Specifications

| Process                   | Behaviour    | Responsibility                                              | Restart | Max Restarts | Period |
|---------------------------|-------------|-------------------------------------------------------------|---------|-------------|--------|
| `ybm_reliability_sup`     | supervisor  | Root supervisor for all reliability processes                | permanent | 10        | 60s    |
| `ybm_health_sup`          | supervisor  | Supervises health aggregation                               | permanent | 5         | 60s    |
| `ybm_health_aggregator`   | gen_server  | Collects dependency status; exposes combined health          | permanent | 5         | 60s    |
| `ybm_dependency_sup`      | supervisor  | Supervises dependency monitors                              | permanent | 10        | 60s    |
| `ybm_pg_monitor`          | gen_server  | Polls PostgreSQL availability every 15s                     | permanent | 5         | 60s    |
| `ybm_redis_monitor`       | gen_server  | Polls Redis availability every 10s                          | permanent | 5         | 60s    |
| `ybm_storage_monitor`     | gen_server  | Polls R2/MinIO availability every 30s                       | permanent | 5         | 60s    |
| `ybm_circuit_sup`         | supervisor  | Supervises circuit breaker processes                        | permanent | 10        | 60s    |
| `ybm_pg_circuit`          | gen_server  | Circuit breaker for PostgreSQL                              | permanent | 5         | 60s    |
| `ybm_redis_circuit`       | gen_server  | Circuit breaker for Redis                                   | permanent | 5         | 60s    |
| `ybm_storage_circuit`     | gen_server  | Circuit breaker for R2/object storage                       | permanent | 5         | 60s    |
| `ybm_recovery_sup`        | supervisor  | Supervises recovery coordination                            | permanent | 5         | 60s    |
| `ybm_recovery_coordinator`| gen_server  | Coordinates recovery when degraded services return          | permanent | 5         | 60s    |

---

## Circuit Breaker Design

### States

```
CLOSED → (failure_count >= threshold) → OPEN → (cooldown elapsed) → HALF_OPEN
HALF_OPEN → (probe succeeds) → CLOSED
HALF_OPEN → (probe fails) → OPEN
```

### Configuration (per dependency)

| Dependency   | Failure Threshold | Cooldown (Open→HalfOpen) | Success Threshold (HalfOpen→Closed) |
|-------------|-------------------|--------------------------|--------------------------------------|
| PostgreSQL  | 5 consecutive     | 30s                      | 2 consecutive successes              |
| Redis       | 5 consecutive     | 15s                      | 2 consecutive successes              |
| R2/Storage  | 3 consecutive     | 30s                      | 1 success                            |

All thresholds are configurable via environment variables.

---

## Failure Scenarios

### PostgreSQL Fails

- Erlang `ybm_pg_monitor` detects failure via probe
- `ybm_pg_circuit` transitions to OPEN
- Erlang publishes `{dependency: postgres, state: open}` on `ybm:reliability:status`
- Rust `/ready` endpoint already returns 503 (existing behavior preserved)
- No retry storms: circuit prevents probes until cooldown expires
- When PostgreSQL recovers: HALF_OPEN probe succeeds → CLOSED → Erlang publishes recovery event

### Redis Fails

- Erlang `ybm_redis_monitor` detects failure
- `ybm_redis_circuit` transitions to OPEN
- Rust features degrade: presence, typing, Pub/Sub, rate limiting
- Persistent messaging remains database-authoritative (existing behavior)
- Recovery follows the same circuit pattern

### R2/Object Storage Fails

- Erlang `ybm_storage_monitor` detects failure
- `ybm_storage_circuit` transitions to OPEN
- Media operations fail gracefully without corrupting message/database state
- Text messaging is unaffected

### Rust Backend Process Crashes

- Erlang remains running independently (separate OS process)
- Erlang detects Rust health event absence and marks backend as degraded
- Clients reconnect using existing WebSocket reconnection protocol
- Erlang does NOT own or manage Rust application state

### Erlang Process Crashes

- OTP supervisor restarts the crashed process per restart strategy
- Unrelated processes remain alive (one_for_one strategy)
- If max restarts exceeded, supervisor escalates to parent
- Rust backend continues operating independently (Erlang is advisory)

---

## Graceful Shutdown

### Shutdown Order

1. Rust receives SIGTERM
2. Rust publishes `{event: shutdown}` on `ybm:reliability:events`
3. Rust stops accepting new HTTP/WebSocket connections
4. Rust drains in-flight requests (existing behavior, 10s timeout)
5. Erlang receives shutdown event, stops monitoring
6. Erlang supervisor tree shuts down in reverse start order
7. Rust closes database/Redis connections
8. Both processes exit

### Erlang Shutdown

- SIGTERM → `ybm_reliability_app:stop/1` called
- Root supervisor calls `terminate/2` on each child
- Each gen_server completes current work in `terminate/2`
- Maximum shutdown timeout: 10s per process, 30s total

---

## Security

### Rust ↔ Erlang Boundary

- Communication via Redis Pub/Sub (same Redis instance, same private network)
- No user tokens, passwords, or PII pass through the reliability channel
- Erlang receives only: dependency names, health status, circuit states
- Redis access uses the same authenticated connection as the existing backend
- In production: Redis connection requires password authentication (existing)
- Network isolation: Erlang service runs on the same Docker network as other services
- No direct database credentials exposed to Erlang (Erlang probes dependencies independently)

---

## Docker / Development

### Expected docker-compose addition

```yaml
reliability:
  build:
    context: ./reliability
    dockerfile: Dockerfile
  container_name: ybm-reliability
  depends_on:
    redis:
      condition: service_healthy
  environment:
    REDIS_URL: redis://redis:6379
    PG_HOST: postgres
    PG_PORT: 5432
    R2_ENDPOINT: http://minio:9000
    PROBE_INTERVAL_PG: 15000
    PROBE_INTERVAL_REDIS: 10000
    PROBE_INTERVAL_STORAGE: 30000
  healthcheck:
    test: ["CMD", "curl", "-f", "http://localhost:8081/health"]
    interval: 10s
    timeout: 5s
    retries: 3
```

### OTP Version

- Erlang/OTP 27 (pinned in Dockerfile)
- Build tool: rebar3
- No external Erlang dependencies unless justified by a specific task

---

## Observability (Phase 15 Minimum)

Phase 15 exposes the minimum signals required to verify fault tolerance. Full dashboards are Phase 16.

| Signal              | Source  | Format                |
|---------------------|---------|----------------------|
| Service health      | Erlang  | HTTP GET /health     |
| Dependency state    | Erlang  | Redis Pub/Sub event  |
| Circuit state       | Erlang  | Redis Pub/Sub event  |
| Restart count       | Erlang  | HTTP GET /health     |
| Degraded state      | Erlang  | Redis Pub/Sub event  |
| Recovery event      | Erlang  | Redis Pub/Sub event  |

---

## Testing Requirements

### Supervisor Tests

- Child process starts successfully
- Child process crashes and restarts
- Repeated crashes trigger supervisor restart limits
- Unrelated child processes remain alive after sibling crash

### Circuit Breaker Tests

- Closed state passes requests through
- Failure threshold transitions to Open
- Open state rejects probes until cooldown
- Half-Open allows single probe
- Successful probe closes circuit
- Failed probe reopens circuit

### Dependency Failure Tests

- PostgreSQL unavailable → monitor detects → circuit opens
- Redis unavailable → monitor detects → circuit opens
- R2 unavailable → monitor detects → circuit opens
- Dependency recovers → circuit closes

### Integration Tests (Rust ↔ Erlang)

- Rust publishes health event → Erlang receives
- Erlang publishes status → verifiable on Redis channel
- Erlang detects degraded state and publishes event
- Dependency recovery detected and reported

### Shutdown Tests

- SIGTERM triggers orderly shutdown
- No new monitoring work accepted during shutdown
- Supervisor tree shuts down cleanly
