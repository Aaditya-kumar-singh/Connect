# ADR-018: Erlang/OTP for Fault-Tolerance and Supervision

| Field | Value |
|-------|-------|
| **ADR ID** | `ADR-018` |
| **Status** | `ACCEPTED` |
| **Date** | 2026-10-04 |
| **Related** | ADR-001, ADR-002, ADR-011, ADR-012 |
| **Supersedes** | ADR-012 (partially; ADR-012 remains valid for application-level Tokio task isolation) |

## Context

YBM Connect is a real-time communication platform with a Rust/Axum modular monolith backend. The backend manages WebSocket connections, real-time messaging, media, presence, calls, and push notifications. It depends on three external services: PostgreSQL (durable state), Redis (ephemeral coordination), and Cloudflare R2/MinIO (media storage).

ADR-012 established "Erlang-inspired supervision" patterns built manually on Tokio primitives. These patterns provide basic panic isolation (Tokio task isolation) and manual worker restart logic. However, they have architectural limitations:

1. **Manual supervision code**: The Rust supervisor must be hand-written, tested, and maintained. OTP provides this as a battle-tested runtime primitive.
2. **No restart intensity tracking**: Tokio has no built-in concept of "restart too many times → escalate." This must be manually implemented and tested.
3. **No structured shutdown coordination**: Each worker uses ad-hoc shutdown channels. OTP provides deterministic `terminate/2` callbacks and shutdown ordering.
4. **Limited failure domain isolation**: A panic in a Tokio task is isolated, but the entire Rust process still shares one allocator, one file descriptor table, and one signal handler. BEAM processes have fully independent heaps.
5. **Circuit breaker state management**: Circuit breakers for external dependencies require stateful, time-aware processes that respond to events from multiple sources. OTP `gen_server` is purpose-built for this pattern.

As the project approaches production hardening (Phases 15-20), stronger fault isolation and structured recovery become important for reliability.

## Decision

Use Erlang/OTP as a dedicated reliability/supervision sidecar runtime alongside the existing Rust backend. Rust remains the primary application runtime for all business logic, API endpoints, data access, and client-facing protocols.

The Erlang/OTP application is responsible for:
- OTP supervision trees with restart strategies and intensity limits
- Dependency health monitoring (PostgreSQL, Redis, R2)
- Circuit breaker state machines (CLOSED/OPEN/HALF_OPEN)
- Recovery detection and coordination
- Aggregated health status

The Erlang/OTP application is NOT responsible for:
- Any business logic (messaging, auth, users, groups, media, calls)
- Serving client-facing APIs
- Storing or accessing application data
- WebSocket or WebRTC protocols

Communication between Rust and Erlang uses Redis Pub/Sub channels, reusing the existing Redis infrastructure.

## Why Erlang/OTP

### BEAM Process Isolation

Each BEAM process has its own heap, stack, and garbage collector. A crash in one process cannot corrupt memory in another process. This is fundamentally stronger isolation than Tokio tasks, which share a global allocator.

### Supervision Trees

OTP supervisors are a first-class runtime primitive, not a library. They provide:
- Declarative child specifications
- Configurable restart strategies (`one_for_one`, `one_for_all`, `rest_for_one`)
- Restart intensity tracking (`MaxRestarts` in `MaxSeconds`)
- Automatic escalation when restart intensity is exceeded
- Deterministic shutdown ordering via `terminate/2`

These behaviors are thoroughly tested across decades of telecom and messaging production use. Reimplementing them in Rust requires significant manual code that must be independently tested and maintained.

### gen_server for Stateful Reliability Processes

Circuit breakers and dependency monitors are stateful, event-driven processes that:
- Maintain internal state (failure counts, timestamps, circuit state)
- Respond to external events (probe results, health reports)
- Perform periodic actions (cooldown timers, probe scheduling)
- Must handle crashes gracefully (supervised restart preserves sibling state)

`gen_server` provides exactly this pattern with standardized callbacks (`init/1`, `handle_call/3`, `handle_cast/2`, `handle_info/2`, `terminate/2`).

### Controlled Failure Propagation

OTP allows precise control over what happens when a process fails:
- The process's supervisor is notified
- The supervisor applies its restart strategy
- Other processes under different supervisors are unaffected
- If the supervisor itself exceeds its restart budget, it crashes, and *its* supervisor handles the escalation

This hierarchical failure propagation is not available in Tokio.

## Why Not Rewrite Rust

The existing Rust backend is functionally complete across 13+ implemented phases:
- Authentication with Argon2id, token rotation, and reuse detection
- 22-table PostgreSQL schema with migrations
- Real-time WebSocket messaging with Redis Pub/Sub routing
- Media upload/download with R2/MinIO
- WebRTC call signaling with TURN credential management
- Push notification delivery
- Presence and typing indicators

A rewrite would:
1. **Increase risk**: Replacing working, tested code with new code introduces regressions
2. **Duplicate work**: All 13 phases of implementation would need to be re-done
3. **Delay delivery**: Months of additional development time
4. **Violate minimal-change principles**: The project's Ponytail rules explicitly prohibit unnecessary rewrites
5. **Provide no guaranteed reliability benefit**: Erlang does not automatically make business logic more correct

The Rust backend's application-level fault tolerance (Tokio task isolation, graceful shutdown, health endpoints) remains valid and is preserved. The Erlang sidecar adds a *complementary* layer for infrastructure-level reliability concerns.

## Alternatives Considered

### 1. Rust/Tokio Only (Status Quo)

Continue with manually implemented supervision patterns per ADR-012.

**Pros**: No new runtime, no deployment complexity, simpler architecture.
**Cons**: Manual supervision code grows with each new worker type. No restart intensity escalation. No structured shutdown ordering. Circuit breaker state management requires custom stateful actor implementation. Higher maintenance burden for reliability code.

**Verdict**: Viable for current scale but increasingly difficult to maintain as the system grows and production reliability requirements tighten.

### 2. Rust Application + Erlang/OTP Reliability Sidecar (Chosen)

**Pros**: OTP supervision is purpose-built for the reliability use case. Clear separation of concerns. Rust handles application logic; Erlang handles reliability. Each runtime is used for what it does best.
**Cons**: Additional runtime to deploy, monitor, and maintain. Redis Pub/Sub boundary adds latency to reliability events. Team must maintain code in two languages. Additional Docker service.

**Verdict**: Chosen. The operational overhead is bounded (single Erlang service, no Erlang clustering) and the reliability benefits are concrete.

### 3. Full Erlang/Elixir Rewrite

Rewrite the entire backend in Erlang or Elixir.

**Pros**: Uniform runtime. OTP supervision for all components. BEAM process isolation everywhere. Hot code reloading.
**Cons**: Discards 13+ phases of working Rust code. Months of rewrite effort. Different ecosystem (SQL, HTTP, WebSocket libraries). No performance benefit for CPU-bound operations (message validation, Argon2id). Violates every Ponytail rule.

**Verdict**: Rejected. The cost is disproportionate to the benefit.

### 4. External Supervisor Only (Kubernetes, systemd)

Rely on Kubernetes or systemd to restart crashed processes. No application-level supervision.

**Pros**: No application code changes. Industry-standard process management.
**Cons**: Process-level restarts are coarse-grained (entire backend restarts, not individual workers). No circuit breaker state. No dependency-aware recovery. No graceful degradation. Kubernetes adds significant operational complexity not justified by current scale.

**Verdict**: Rejected for Phase 15. Kubernetes may be appropriate for Phase 19 (Deployment) but does not replace application-level fault tolerance.

## Consequences

### Positive

- OTP supervision provides proven, tested reliability primitives
- Circuit breakers prevent cascading failures to external dependencies
- Independent failure detection even when Rust backend is unresponsive
- Clear architectural boundary between application logic and reliability logic
- Erlang sidecar can be deployed/restarted independently of Rust backend

### Negative / Costs

- **Additional runtime**: Erlang/OTP 27 must be installed, built, and deployed
- **Operational complexity**: One additional Docker service to monitor
- **Two-language codebase**: Team must maintain Erlang alongside Rust
- **IPC latency**: Redis Pub/Sub adds milliseconds to reliability event delivery (acceptable for advisory signals)
- **Testing complexity**: Both Erlang and Rust test suites must pass
- **Development setup**: Docker Compose adds one more service

### Mitigations

- Erlang codebase is small and focused (reliability only, no business logic)
- Erlang service is advisory: Rust operates independently if Erlang is unavailable
- Redis Pub/Sub is already a project dependency (zero new infrastructure)
- OTP application structure is minimal and follows standard conventions

## Reconsider When

- If the Erlang service becomes a maintenance burden disproportionate to its reliability benefit
- If Kubernetes is adopted and its health/restart capabilities subsume the Erlang sidecar's responsibilities
- If Rust ecosystem develops a mature, production-tested supervision framework
