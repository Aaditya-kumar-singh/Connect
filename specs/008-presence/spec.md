# Phase 8: Presence

## Scope

Implement online/offline presence and durable last-seen timestamps for authenticated WebSocket users.

### In scope
- Redis presence key presence:{user_id} with 60-second TTL.
- Presence becomes online when the first active WebSocket connection is established.
- Existing WebSocket heartbeat renews presence.
- Presence becomes offline when the final active connection closes or the heartbeat registry expires.
- users.last_seen_at is updated when the user transitions offline.
- presence.update is delivered to contacts and active conversation members.
- Cross-instance presence delivery through the existing Redis Pub/Sub subscriber.
- Multi-device support: additional connections do not emit duplicate online/offline transitions.
- Focused unit tests and an ignored live integration test.

### Out of scope
- Typing indicators.
- Privacy controls.
- Presence REST endpoint.
- Groups.
- Push notifications.
- Calls.
- Changes to message or receipt behavior.

## Requirements

1. The first active WebSocket connection for a user sets presence:{user_id} to online with a 60-second TTL.
2. Additional active connections do not generate duplicate online transitions.
3. The existing heartbeat renews the presence TTL.
4. If the presence key disappears while a connection remains active, the next heartbeat restores online state and emits an online update.
5. When the final active connection closes, the presence key is removed and users.last_seen_at is set to the server timestamp.
6. Closing one of several active connections does not emit offline or update last_seen_at.
7. presence.update contains user_id, status, and last_seen_at.
8. Presence updates are sent to contacts and users sharing an active conversation.
9. Local connections receive updates directly; Redis Pub/Sub carries updates to other instances.
10. Redis failure must not crash the WebSocket process; presence remains best-effort while durable messaging behavior is unchanged.
11. No new dependency is required.

## Acceptance criteria

- Connecting a previously offline user produces one online presence update.
- Multiple devices remain online until the final connection closes.
- Heartbeats keep the Redis presence key alive.
- Final disconnect produces one offline update and persists last_seen_at.
- Existing message, receipt, authentication, and WebSocket tests remain passing.
