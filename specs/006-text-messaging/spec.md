# Phase 6 — Text Messaging

## Scope
Implement the approved text-messaging lifecycle:
- messages persistence
- message history with cursor pagination
- WebSocket message.send / message.ack / message.new
- client_message_id idempotency
- reply_to_message_id
- message edit within 15 minutes
- soft delete
- emoji reactions add/remove
- REST message forwarding across direct conversations
- same-instance delivery through the WebSocket connection manager
- cross-instance delivery through Redis pub/sub
- conversation updated_at maintenance
- authorization through active conversation membership and sender ownership

## Non-goals
Do not implement delivery/read receipts, presence, typing indicators, groups, media upload/attachments, push notifications, or calls.

## Rules
- PostgreSQL is authoritative and messages are persisted before ACK.
- client_message_id is unique per conversation.
- text content is 1–4096 UTF-8 characters.
- content_type for Phase 6 is text only.
- reply target must exist in the same conversation and be active/non-deleted.
- only the original sender may edit/delete.
- edits are allowed for 15 minutes after creation.
- deletion sets deleted_at and clears content.
- reactions are keyed by message/user/emoji and are idempotent.
- all message operations require active membership.
- Redis failure must not cause message loss; persistence and ACK remain successful, while real-time cross-instance delivery may degrade.

## WebSocket events
Client:
- message.send
- message.edit
- message.delete
- message.react

Server:
- message.ack
- message.new
- message.edited
- message.deleted
- message.reaction
- error

## REST
- GET /api/v1/conversations/{id}/messages
- POST /api/v1/messages/forward

## Acceptance
1. A member can send text and receives ACK only after DB commit.
2. Other active connections receive message.new.
3. Duplicate client_message_id returns the existing persisted message without duplication.
4. History is cursor paginated by created_at/id descending.
5. Reply validation prevents cross-conversation replies.
6. Sender can edit within 15 minutes and receives/broadcasts message.edited.
7. Sender can soft-delete and receives/broadcasts message.deleted.
8. Reaction add/remove is idempotent and broadcast.
9. Forward creates a new message in the destination conversation referencing the source.
10. Unauthorized/non-member operations fail without data leakage.
11. No Phase 7+ functionality is introduced.
