# Implementation Plan: Observability & Production Metrics

**Branch**: `016-observability` | **Date**: 2026-10-04 | **Spec**: `spec.md`

## Summary

Add a small in-process Prometheus registry to the Rust backend and instrument the existing HTTP/WebSocket boundaries. Keep the implementation inside the existing modular monolith; no new service is required.

1. Add the `prometheus` crate.
2. Create `infrastructure/metrics.rs` containing the registry and bounded metric definitions.
3. Add metrics to `AppState`.
4. Add `GET /metrics` and a lightweight HTTP metrics middleware.
5. Instrument WebSocket connection lifecycle and bounded event types.
6. Expose SQLx pool statistics as gauges when the metrics endpoint is scraped.
7. Improve `TraceLayer` so request IDs are included in structured spans.
8. Add focused unit/integration tests.
9. Update observability documentation and environment/reference docs only where required.

## Architecture

- Metrics are local process state and are never persisted.
- `AppState` owns an `Arc<Metrics>` so all modules share the same registry.
- The metrics endpoint only reads memory and SQLx pool counters; it performs no network I/O.
- HTTP labels are method/path/status. Paths are normalized to route templates where practical; otherwise only known API prefixes are used and raw query strings are excluded.
- WebSocket event labels come from a fixed allowlist.
- Pool gauges are updated immediately before metric collection rather than maintained by background polling.

## Dependencies

- Existing Rust/Axum/Tokio/SQLx/tracing stack.
- `prometheus = 0.14`.

## Verification

1. `cargo fmt --check`
2. `cargo check`
3. `cargo test --lib --no-fail-fast`
4. `cargo test --test integration --no-fail-fast`
5. `cargo clippy --all-targets --all-features -- -D warnings`
6. `git diff --check`
7. Manual `GET /metrics` verification when the backend is running.
