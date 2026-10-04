# Tasks: Observability & Production Metrics

**Spec**: `spec.md` | **Plan**: `plan.md` | **Date**: 2026-10-04

## Phase 1 — Metrics Foundation

- [x] T001 Add `prometheus = 0.14` and refresh Cargo.lock.
- [x] T002 Implement `backend/src/infrastructure/metrics.rs` with bounded HTTP, WebSocket, and DB-pool metrics.
- [x] T003 Add `Arc<Metrics>` to `AppState` and initialize it once during startup.

## Phase 2 — HTTP Observability

- [x] T004 Add `GET /metrics`.
- [x] T005 Add HTTP request counter/latency middleware without logging request bodies or query strings.
- [x] T006 Include request ID in the HTTP tracing span.

## Phase 3 — WebSocket Observability

- [x] T007 Instrument WebSocket connection accepted/cleanup lifecycle.
- [x] T008 Instrument bounded WebSocket received/sent event counters.

## Phase 4 — Pool Metrics & Tests

- [x] T009 Export SQLx pool active/idle/max gauges.
- [x] T010 Add metrics endpoint and middleware tests.
- [x] T011 Add WebSocket metric unit coverage for bounded event labels.

## Phase 5 — Verification & Documentation

- [x] T012 Run fmt, check, tests, clippy, and diff audit.
- [x] T013 Update `docs/15-observability/logging.md` to reflect implemented scope.
- [x] T014 Mark Phase 16 implementation status in roadmap/spec documentation.
