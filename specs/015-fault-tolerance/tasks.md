# Tasks: Fault Tolerance & Reliability (Erlang/OTP Supervision Layer)

**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md) | **Date**: 2026-10-04

---

## Task 1: Erlang/OTP Project Skeleton

**Status**: `TODO`
**Dependencies**: None
**Files**:
- `reliability/rebar.config`
- `reliability/src/ybm_reliability.app.src`
- `reliability/src/ybm_reliability_app.erl`
- `reliability/src/ybm_reliability_sup.erl`
- `reliability/README.md`

**Description**: Create the minimal Erlang/OTP application structure with rebar3. Implement `ybm_reliability_app` (application behaviour) and `ybm_reliability_sup` (root supervisor with `one_for_one` strategy). The application should start, supervise an empty child list, and stop cleanly. Verify with `rebar3 compile`.

**Acceptance Criteria**:
- `rebar3 compile` succeeds
- Application starts and stops without error
- Root supervisor uses `one_for_one` strategy with `{10, 60}` restart intensity

---

## Task 2: Redis Pub/Sub Client

**Status**: `TODO`
**Dependencies**: Task 1
**Files**:
- `reliability/rebar.config` (add `eredis` dependency)
- `reliability/src/ybm_redis_pubsub.erl`
- `reliability/test/ybm_redis_pubsub_tests.erl`

**Description**: Implement `ybm_redis_pubsub` gen_server that connects to Redis and subscribes to `ybm:reliability:events`. Provide API to publish messages on `ybm:reliability:status`. Handle Redis connection failures gracefully (log and retry).

**Acceptance Criteria**:
- Connects to Redis on startup
- Subscribes to the events channel
- Can publish messages to status/commands channels
- Handles Redis unavailability without crashing
- eunit tests for message encoding/decoding

---

## Task 3: Generic Circuit Breaker

**Status**: `TODO`
**Dependencies**: Task 1
**Files**:
- `reliability/src/ybm_circuit_breaker.erl`
- `reliability/test/ybm_circuit_breaker_tests.erl`

**Description**: Implement `ybm_circuit_breaker` as a gen_server parameterized by: dependency name, failure threshold, cooldown duration, success threshold. Implement the CLOSED → OPEN → HALF_OPEN state machine. Expose API: `report_success/1`, `report_failure/1`, `get_state/1`.

**Acceptance Criteria**:
- State transitions match spec: CLOSED→OPEN on threshold, OPEN→HALF_OPEN on cooldown, HALF_OPEN→CLOSED on success, HALF_OPEN→OPEN on failure
- Configurable thresholds per instance
- eunit tests for all state transitions
- `get_state/1` returns current state, failure count, last transition time

---

## Task 4: Circuit Breaker Supervisor

**Status**: `TODO`
**Dependencies**: Task 3
**Files**:
- `reliability/src/ybm_circuit_sup.erl`

**Description**: Implement `ybm_circuit_sup` supervisor that starts three `ybm_circuit_breaker` children (postgres, redis, storage) with configuration from application environment. Add as child of `ybm_reliability_sup`.

**Acceptance Criteria**:
- Three circuit breaker children start under supervision
- Each has independent configuration (thresholds from app env)
- `one_for_one` strategy: crash of one circuit breaker does not affect others
- Supervisor restarts crashed children

---

## Task 5: Dependency Monitors

**Status**: `TODO`
**Dependencies**: Task 2, Task 4
**Files**:
- `reliability/src/ybm_dependency_sup.erl`
- `reliability/src/ybm_pg_monitor.erl`
- `reliability/src/ybm_redis_monitor.erl`
- `reliability/src/ybm_storage_monitor.erl`
- `reliability/test/ybm_monitor_tests.erl`

**Description**: Implement three dependency monitor gen_servers. Each periodically probes its dependency:
- `ybm_pg_monitor`: TCP connect to PostgreSQL port (configurable interval, default 15s)
- `ybm_redis_monitor`: Redis PING (configurable interval, default 10s)
- `ybm_storage_monitor`: HTTP HEAD to R2/MinIO endpoint (configurable interval, default 30s)

On probe result, call `ybm_circuit_breaker:report_success/1` or `report_failure/1` for the corresponding circuit breaker. Implement `ybm_dependency_sup` as supervisor. Add as child of `ybm_reliability_sup`.

**Acceptance Criteria**:
- Each monitor probes at configurable intervals
- Probe success/failure correctly reports to circuit breaker
- Monitors handle probe timeout without crashing
- eunit tests with mocked probes

---

## Task 6: Health Aggregator

**Status**: `TODO`
**Dependencies**: Task 4, Task 5
**Files**:
- `reliability/src/ybm_health_sup.erl`
- `reliability/src/ybm_health_aggregator.erl`
- `reliability/src/ybm_health_httpd.erl`

**Description**: Implement `ybm_health_aggregator` gen_server that queries all circuit breaker states and computes an aggregated health status. Expose the exact `GET /health` endpoint on `:8081` using Erlang's built-in Inets `httpd` callback module. Implement `ybm_health_sup` supervisor. Add as child of `ybm_reliability_sup`.

**Acceptance Criteria**:
- `GET :8081/health` returns JSON with overall status and per-dependency circuit states
- Status is `healthy` when all circuits closed, `degraded` when any circuit open
- No external HTTP framework dependency (use `inets`/`httpd` or `cowboy` only if justified)

---

## Task 7: Recovery Coordinator

**Status**: `TODO`
**Dependencies**: Task 4, Task 5
**Files**:
- `reliability/src/ybm_recovery_sup.erl`
- `reliability/src/ybm_recovery_coordinator.erl`

**Description**: Implement `ybm_recovery_coordinator` gen_server that subscribes to circuit breaker state changes. When a circuit transitions from OPEN/HALF_OPEN → CLOSED, publish a recovery event on `ybm:reliability:status` via `ybm_redis_pubsub`. When a circuit opens, publish a degradation event. Implement `ybm_recovery_sup` supervisor. Add as child of `ybm_reliability_sup`.

**Acceptance Criteria**:
- Publishes recovery event when circuit closes after being open
- Publishes degradation event when circuit opens
- Events are published on the correct Redis Pub/Sub channel
- Does not generate events on startup for initially-closed circuits

---

## Task 8: Supervisor Tests (Common Test)

**Status**: `TODO`
**Dependencies**: Task 4, Task 5, Task 6, Task 7
**Files**:
- `reliability/test/ybm_supervisor_SUITE.erl`

**Description**: Write Common Test suite verifying OTP supervision behavior:
- Application starts all supervisors and children
- Killing a child process results in restart
- Repeated crashes (beyond max intensity) escalate to parent supervisor
- Sibling processes remain alive after one sibling crashes
- Application stops cleanly without errors

**Acceptance Criteria**:
- All CT tests pass with `rebar3 ct`
- Tests cover start, crash-restart, max-restart-escalation, sibling isolation, clean stop

---

## Task 9: Dockerfile and Docker Compose

**Status**: `TODO`
**Dependencies**: Task 1
**Files**:
- `reliability/Dockerfile`
- `docker-compose.yml` (add `reliability` service)

**Description**: Create a multi-stage Dockerfile for the Erlang reliability application (OTP 27, rebar3 compile + release). Add the `reliability` service to `docker-compose.yml` with:
- Depends on `redis` (service_healthy)
- Environment variables for Redis URL, PG host/port, R2 endpoint, probe intervals
- Health check against `:8081/health`

**Acceptance Criteria**:
- `docker compose build reliability` succeeds
- `docker compose up reliability` starts the service
- Service health check passes when Redis is healthy
- Service starts after Redis is ready

---

## Task 10: Rust Event Publishing

**Status**: `TODO`
**Dependencies**: Task 2
**Files**:
- `backend/src/infrastructure/reliability_events.rs`
- `backend/src/infrastructure/mod.rs` (add module)

**Description**: Add a minimal `reliability_events` module to the Rust backend that publishes JSON events on Redis Pub/Sub channel `ybm:reliability:events`. Events include: `startup`, `shutdown`, `health_check_result`. Publish `startup` event on server start and `shutdown` event during graceful shutdown. This module is purely additive and does not modify existing Rust behavior.

**Acceptance Criteria**:
- `startup` event published when Rust backend starts
- `shutdown` event published when Rust backend receives SIGTERM
- Events are valid JSON with `{event, timestamp, source: "rust"}`
- Existing Rust tests continue to pass
- `cargo clippy -- -D warnings` passes

---

## Task 11: Graceful Shutdown Coordination

**Status**: `TODO`
**Dependencies**: Task 7, Task 10
**Files**:
- `reliability/src/ybm_reliability_app.erl` (update stop/1)
- `reliability/src/ybm_redis_pubsub.erl` (subscribe to shutdown event)

**Description**: When the Erlang Redis Pub/Sub client receives a `{event: shutdown}` message from Rust on `ybm:reliability:events`, initiate orderly shutdown of the Erlang application. Stop dependency monitors (stop probing), then stop circuit breakers, then stop health aggregator. Ensure the Erlang application also handles SIGTERM independently.

**Acceptance Criteria**:
- Rust shutdown event triggers Erlang shutdown
- SIGTERM directly to Erlang container also triggers clean shutdown
- No monitoring probes fire after shutdown initiated
- Supervisor tree stops in reverse start order

---

## Task 12: Environment and Configuration Documentation

**Status**: `TODO`
**Dependencies**: Task 9
**Files**:
- `.env.example` (add reliability-related variables)
- `reliability/README.md` (update with setup/run instructions)

**Description**: Document all new environment variables for the reliability service. Update `.env.example` with the new variables (commented out, with descriptions). Update `reliability/README.md` with development setup instructions.

**Acceptance Criteria**:
- `.env.example` documents all new environment variables
- `reliability/README.md` explains how to build, run, and test the Erlang application
- No undocumented configuration

---

## Task 13: Update AGENTS.md and GEMINI.md

**Status**: `DONE`
**Dependencies**: Task 1
**Files**:
- `AGENTS.md`
- `GEMINI.md`

**Description**: Add a section to both AI instruction files clarifying:
- Rust remains the primary backend language
- Erlang/OTP is permitted only for approved reliability responsibilities defined in Phase 15 spec
- AI agents must not introduce Erlang into application/business logic
- AI agents must not rewrite Rust modules in Erlang
- Erlang dependencies require architectural justification
- Ponytail rules apply equally to Erlang code
- The Rust/Erlang boundary follows the Phase 15 specification

**Acceptance Criteria**:
- Both files updated with Erlang/OTP guidance
- Guidance is clear and actionable for AI agents
- No existing rules removed or weakened

---

## Task 14: Update Constitution

**Status**: `DONE`
**Dependencies**: Task 1
**Files**:
- `.specify/memory/constitution.md`

**Description**: Update the Technical Constraints & Stack Requirements section to reflect that Erlang/OTP 27 is used for dedicated reliability/fault-tolerance responsibilities where explicitly justified by architecture. Rust remains the primary backend language. Bump version to 1.1.0.

**Acceptance Criteria**:
- Constitution accurately reflects the dual-runtime architecture
- Rust is clearly identified as the primary backend language
- Erlang/OTP role is scoped to reliability responsibilities
- Version bumped

---

## Task 15: Update Roadmap

**Status**: `DONE`
**Dependencies**: None
**Files**:
- `docs/22-product/roadmap.md`

**Description**: Update Phase 15 in the roadmap from "Supervisor tree, circuit breakers, graceful shutdown" to describe the Erlang/OTP reliability layer. Document what each runtime owns.

**Acceptance Criteria**:
- Phase 15 description identifies Erlang/OTP explicitly
- Rust and Erlang responsibilities are clearly delineated
- No vague marketing language

---

## Task 16: Create ADR-018

**Status**: `DONE`
**Dependencies**: None
**Files**:
- `docs/02-architecture/architecture-decisions/ADR-018-erlang-otp-fault-tolerance.md`

**Description**: Create ADR-018 documenting the decision to use Erlang/OTP for fault tolerance. Include: context, existing architecture, decision, why OTP, why not rewrite Rust, alternatives considered (Rust-only, Rust+Erlang, full Erlang rewrite, Kubernetes-only), consequences.

**Acceptance Criteria**:
- ADR follows the existing ADR format and is consistent with ADR-001 through ADR-018
- Factual comparison without arbitrary scores
- Documents operational consequences (additional runtime, deployment, monitoring, IPC)

---

## Task 17: Update Architecture Documentation

**Status**: `DONE`
**Dependencies**: Task 16
**Files**:
- `docs/02-architecture/architecture-overview.md` (update deployment units, container diagram)
- `docs/02-architecture/container-architecture.md` (add reliability service)
- `docs/02-architecture/failure-domains.md` (add Erlang failure domain)
- `docs/14-fault-tolerance/fault-tolerance.md` (update to reflect Erlang/OTP)
- `docs/MASTER-ENGINEERING-GUIDE.md` (update sections 2, 9)

**Description**: Update architecture documentation to reflect the Erlang/OTP reliability sidecar. Add the reliability service to deployment units (now 5 instead of 4). Update container and failure domain diagrams. Update the fault tolerance reference document. Update the Master Engineering Guide technology stack and fault tolerance sections.

**Acceptance Criteria**:
- Deployment units list includes Erlang reliability service
- Container diagram shows the reliability service
- Failure domain map includes Erlang as a separate failure domain
- Fault tolerance document references OTP supervision instead of Rust-only patterns
- Master Engineering Guide reflects the updated architecture
- No unrelated documentation changes

---

## Task 18: Final Verification and Diff Audit

**Status**: `TODO`
**Dependencies**: All previous tasks
**Files**: None (verification only)

**Description**: Run final verification:
1. `rebar3 compile` — Erlang compiles
2. `rebar3 eunit` — unit tests pass
3. `rebar3 ct` — Common Test passes
4. `cargo check` — Rust compiles
5. `cargo clippy -- -D warnings` — no warnings
6. `cargo test` — existing Rust tests pass
7. `git diff --check` — no whitespace issues
8. `git diff` — review all changes, confirm no unrelated modifications
9. Verify spec/plan/tasks consistency

**Acceptance Criteria**:
- All compilation and test commands pass
- No unrelated files modified
- Implementation matches specification
