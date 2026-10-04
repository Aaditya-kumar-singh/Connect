# Phase 7: Message Receipts

## Scope

Implement durable delivery and read receipts for Phase 6 text messages.

### In scope
- message_receipts PostgreSQL table.
- Client WebSocket event message.delivered.
- Client WebSocket event message.read.
- Server WebSocket event message.status.
- Delivery receipt persistence, idempotency, and sender notification.
- Cursor-based read receipts using conversation_members.last_read_message_id.
- Multi-device semantics: a user's receipt is satisfied when any active device reports delivery/read.
- Authorization for conversation membership.
- Per-event rate limiting.
- Focused unit tests and an ignored live integration test.

### Out of scope
- Presence, typing, groups, media, push notifications, calls.
- Per-device receipt rows.
- New REST receipt endpoints.
- Unread-count APIs beyond maintaining the existing read cursor.
- Changes to Phase 6 message behavior.

## Functional requirements

1. message.delivered accepts { message_ids: [uuid, ...] }.
2. Every referenced message must exist and belong to a conversation where the authenticated user is an active member.
3. A delivery receipt stores the first delivery timestamp for (message_id,user_id). Repeating the same acknowledgement is a no-op.
4. Delivery receipts are only meaningful for messages sent by another user.
5. After a new delivery receipt is committed, the message sender receives message.status with status=delivered, message_id, user_id, and the receipt timestamp.
6. message.read accepts conversation_id and last_read_message_id.
7. The target message must belong to the supplied conversation and the authenticated user must be an active member.
8. Read is cursor-based: all messages from other users in that conversation up to and including the target are marked read.
9. Read receipts are idempotent and monotonic. A stale cursor never moves last_read_message_id backwards.
10. The existing conversation_members.last_read_message_id is updated to the newest accepted cursor.
11. For every newly created read receipt, the corresponding message sender receives message.status with status=read.
12. A read operation never generates read receipts for messages authored by the reading user.
13. Receipt writes and the conversation read cursor update are committed before status events are published.
14. Redis/pub/sub failure must not roll back or lose a committed receipt; status delivery is best-effort.
15. message.delivered is limited to 60 events/minute and message.read to 30 events/minute, plus the existing 120 total WebSocket events/minute limit.
16. Receipt handlers require an authenticated WebSocket session and request_id.
17. No new dependencies are required.

## Protocol

Client event message.delivered:
{ type: "message.delivered", request_id: "uuid", payload: { message_ids: ["uuid1", "uuid2"] } }

Client event message.read:
{ type: "message.read", request_id: "uuid", payload: { conversation_id: "uuid", last_read_message_id: "uuid" } }

Server event message.status:
{ type: "message.status", request_id: "uuid", timestamp: "server timestamp", payload: { message_id: "uuid", status: "delivered|read", user_id: "uuid", timestamp: "receipt timestamp" } }

## Acceptance criteria

- A recipient can acknowledge one or more received messages and the sender gets delivered status after persistence.
- Duplicate delivery acknowledgements produce no duplicate receipt or status side effect.
- A recipient can advance a conversation read cursor and the sender receives read status for newly-read messages.
- Replaying an old read cursor never moves the cursor backwards or produces duplicate read side effects.
- Unauthorized message/conversation references are rejected.
- Receipt persistence survives Redis failure.
- Existing Phase 6 message behavior remains unchanged.
