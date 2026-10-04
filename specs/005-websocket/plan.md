# Phase 5 Plan

## Architecture
WebSocket transport lives in `backend/src/websocket/`.
- `protocol.rs`: frame types and handshake payloads.
- `manager.rs`: local connection registry and outbound channels.
- `connection.rs`: one upgraded socket lifecycle, heartbeat and cleanup.
- `handlers.rs`: upgrade/auth/device validation.
- `pubsub.rs`: instance Redis subscriber foundation.
- `mod.rs`: module exports and shared constants.

AppState owns the connection manager and instance identity so handlers and background tasks share one lifecycle.

## Authentication
Reuse `AuthenticatedSession` and `auth::service::authenticate`; do not create a second token system. Validate X-Device-Id against the authenticated session.

## Registry
Local registry uses bounded Tokio channels for each connection. Redis stores connection metadata under `connections:{user_id}` with a 300-second TTL. Heartbeats refresh the user registry TTL; disconnect removes the exact member and deletes the set when empty.

## Pub/Sub
Use a dedicated Redis connection for Pub/Sub. Subscribe to `instance:{instance_id}`. Messages are currently logged/ignored as no Phase 6 event routing is approved yet. Redis subscriber failures are retried with bounded backoff and do not crash the server.

## Verification
Run cargo fmt check, cargo check, clippy with warnings denied, unit tests, and git diff --check. Live WebSocket/Redis integration tests remain ignored when infrastructure is unavailable.
