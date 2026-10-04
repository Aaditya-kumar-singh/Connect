# Phase 5 — Real-Time WebSocket Infrastructure

## Scope
Implement only:
- WebSocket upgrade at GET /ws/connect
- access-token authentication during upgrade
- required device identification and session/device binding
- per-connection tracking with multiple connections per user/device
- Redis-backed connection registry with TTL refresh
- server.connected handshake
- auth.ping -> auth.pong heartbeat
- 60-second heartbeat timeout
- graceful close and registry cleanup
- Redis pub/sub subscriber foundation for future cross-instance routing

## Explicit non-goals
Do not implement messages, presence, typing, calls, conversation sync, media, push notifications, or Phase 6+ event handlers.

## Protocol
- Authorization: Bearer <access token>, or ?token=<access token>.
- X-Device-Id is required and must equal the authenticated session device_id.
- Invalid auth/device mismatch closes with 4001.
- Missing device ID closes with 4008.
- Success immediately sends server.connected with session_id, user_id, device_id and server_time.
- Client auth.ping receives auth.pong.
- No client activity for 60 seconds closes with 4000.
- WebSocket control Ping/Pong frames are handled as transport keepalive.
- Normal graceful close uses code 1000.
- Disconnect never revokes the authentication session.

## Connection registry
- Local manager tracks connection_id -> user/session/device + outbound sender.
- Multiple connections per user/device are allowed.
- Redis set connections:{user_id} contains instance_id:connection_id:device_id.
- Registry TTL is 5 minutes and is renewed by heartbeat.
- Redis is coordination only; PostgreSQL remains authoritative.

## Redis pub/sub
- Dedicated async subscriber task per backend instance.
- Subscribe to instance:{instance_id}.
- Subscriber failure must not terminate the HTTP server.
- Shutdown must stop the subscriber cleanly.

## Acceptance criteria
1. Authenticated client can upgrade.
2. Invalid/missing auth is rejected with documented close codes.
3. Device ID is required and session-bound.
4. server.connected is emitted immediately.
5. auth.ping receives auth.pong.
6. Idle connections close after 60 seconds.
7. Multiple connections are tracked independently.
8. Disconnect removes registry entry.
9. Redis registry TTL is bounded.
10. Subscriber starts without blocking HTTP startup and shuts down cleanly.
11. No Phase 6+ messaging behavior is introduced.
