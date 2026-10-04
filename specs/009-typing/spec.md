# Phase 9 — Typing Indicators

## Goal
Provide short-lived real-time typing indicators for conversation members.

## In Scope
- WebSocket \`typing.start\` and \`typing.stop\`.
- Conversation membership authorization.
- Redis key \`typing:{conversation_id}:{user_id}\` with a 5-second TTL.
- \`typing.start\` refreshes the short-lived state.
- \`typing.stop\` removes the state and broadcasts only when active state existed.
- Server-to-client payload includes conversation ID, user ID, and display name.
- Local delivery through the existing connection manager.
- Cross-instance delivery through the existing Redis Pub/Sub user channel.
- \`typing.start\` limited to once per 2 seconds per user/conversation.
- Existing total WebSocket event limit remains enforced.

## Out of Scope
- Typing REST endpoints.
- Persistent typing records or database migrations.
- Group-specific behavior beyond the existing active conversation-member model.
- Push notifications.
- Presence changes.
- Read/delivery receipts.
- Calls or media.

## Protocol

Client request:

\`typing.start\`
\`\`\`json
{"conversation_id":"uuid"}
\`\`\`

\`typing.stop\`
\`\`\`json
{"conversation_id":"uuid"}
\`\`\`

Server event to other members:

\`\`\`json
{
  "type": "typing.start",
  "request_id": "uuid",
  "timestamp": "ISO-8601",
  "payload": {
    "conversation_id": "uuid",
    "user_id": "uuid",
    "display_name": "Alice"
  }
}
\`\`\`

\`typing.stop\` uses the same payload shape.

## Behavior
1. A member sends \`typing.start\`.
2. The server verifies active membership.
3. The server stores/refreshes the Redis typing key for 5 seconds.
4. The server broadcasts the event to other active conversation members.
5. The client sends \`typing.stop\` after its 2-second debounce or when sending the message.
6. Redis expiry removes stale typing state if no further start arrives.
7. The existing client-side timeout is responsible for hiding an indicator when no further event arrives; no persistent server-side typing record is created.

## Failure Handling
Redis errors return an internal/service error for the typing operation, while existing message persistence behavior remains independent.

## Acceptance Criteria
- Unauthorized/non-member users cannot create or stop typing state.
- Typing state expires after 5 seconds without renewal.
- Start events cannot be sent more often than once per 2 seconds for the same user/conversation.
- Other members receive start/stop events locally and across instances.
- Sender does not receive their own typing event.
- No PostgreSQL schema changes are required.
