# Phase 5 Tasks

- [x] T001 Add websocket module and protocol types.
- [x] T002 Add shared connection manager to AppState.
- [x] T003 Add Redis connection registry helpers with bounded TTL.
- [x] T004 Implement authenticated WebSocket upgrade and device binding.
- [x] T005 Implement server.connected handshake.
- [x] T006 Implement auth.ping/auth.pong heartbeat and 60s idle timeout.
- [x] T007 Implement graceful disconnect and registry cleanup.
- [x] T008 Add Redis instance pub/sub subscriber foundation.
- [x] T009 Register GET /ws/connect route without changing existing REST behavior.
- [x] T010 Add focused unit tests for protocol/validation/manager behavior.
- [ ] T011 Add ignored live integration test for WebSocket + Redis when services are available.
- [x] T012 Run fmt, check, clippy, tests, diff audit and confirm Phase 6+ scope is untouched.

T011 remains open because the development environment has no verified live PostgreSQL/Redis service path and no WebSocket integration client dependency was added solely for an unexecutable test. Unit/compile coverage is active; live infrastructure verification remains an environment task.
