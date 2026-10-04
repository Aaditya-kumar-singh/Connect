# Phase 8 Implementation Plan

## Architecture

Use a small presence module behind the existing WebSocket lifecycle:

WebSocket connection lifecycle -> presence service -> Redis / PostgreSQL -> existing ConnectionManager + Redis Pub/Sub.

PostgreSQL remains authoritative for last_seen_at. Redis is ephemeral presence state only.

## State

- Redis key: presence:{user_id}
- Value: online
- TTL: 60 seconds
- Existing connections:{user_id} set identifies active devices/connections.

The connection set is already renewed by the WebSocket lifecycle. A Redis Lua script removes a closing connection and atomically checks whether it was the final connection before deleting the presence key.

## Events

Server event:
presence.update

Payload:
{ user_id, status: online|offline, last_seen_at }

Online is emitted only on the first active connection or when a missing presence key is restored.

Offline is emitted only after the final active connection is removed.

## Implementation

1. Add presence module and payload/frame generation.
2. Add Redis online/heartbeat/final-disconnect operations.
3. Update WebSocket connect, heartbeat, and cleanup lifecycle hooks.
4. Extend existing Redis Pub/Sub subscriber to presence:user:*.
5. Broadcast updates to contacts and active conversation members.
6. Add focused tests for payload/state helper behavior.
7. Add ignored live integration test.
8. Run fmt/check/clippy/tests/diff audit.

No Phase 9 typing behavior is introduced.
