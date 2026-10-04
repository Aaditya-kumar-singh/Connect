# Phase 7 Implementation Plan

## Architecture

Keep the existing modular-monolith layering:

WebSocket transport -> messaging receipt service -> messaging repository -> PostgreSQL.

The existing message_handlers::deliver_to_conversation remains the transport for status events. Redis is used only for cross-instance event routing; PostgreSQL remains authoritative.

## Data model

Add migration 014_create_message_receipts:
- message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE
- user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE
- delivered_at TIMESTAMPTZ NULL
- read_at TIMESTAMPTZ NULL
- primary key (message_id,user_id)
- index (user_id,message_id)

No schema changes are needed to conversation_members because Phase 6 already added last_read_message_id.

## Implementation

1. Add receipt request/status payload types to messaging/models.rs.
2. Add receipt repository functions for idempotent delivery and monotonic read-cursor updates.
3. Add receipt service functions for authorization, validation, transaction boundaries, and business rules.
4. Add receipt handlers that create message.status frames and notify the message sender through the existing delivery helper.
5. Extend the existing WebSocket dispatcher with message.delivered and message.read.
6. Add event-specific rate limits while preserving the existing total event limit.
7. Add focused unit tests for validation and cursor behavior.
8. Add an ignored live integration test covering delivery/read persistence and status semantics.
9. Update Phase 7 tasks after implementation.

## Transaction rules

Delivery:
- Begin transaction.
- Validate all message IDs and active membership.
- Insert/update receipts idempotently.
- Commit.
- Publish status events only for newly created delivery receipts.

Read:
- Begin transaction.
- Validate active membership and target message ownership by conversation.
- Compare target cursor with the member's current cursor using (created_at,id).
- If stale/equal, commit/no-op.
- Otherwise mark all qualifying messages authored by other users up to the target as read, preserving existing read_at.
- Update conversation_members.last_read_message_id.
- Commit.
- Publish status events only for newly created read receipts.

## Failure behavior

Receipt persistence is durable even if Redis or a recipient/sender WebSocket is unavailable. Status delivery is best-effort; clients can reconcile from PostgreSQL-backed sync/history in later phases.

No Phase 8+ behavior is introduced.
