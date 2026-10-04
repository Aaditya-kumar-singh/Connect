# Phase 6 Plan

## Architecture
Use backend/src/messaging/ with models, validation, repository, service, and handlers.
The existing WebSocket connection manager remains the transport registry. Messaging services produce protocol frames; the connection layer sends them to local connections.

## Persistence
Create messages, message_edits, and message_reactions. Add the approved FK from conversation_members.last_read_message_id to messages.id only after messages exists. Add indexes required by the approved query patterns.

## Delivery
For same-instance recipients, send directly through ConnectionManager. For cross-instance recipients, publish a user-targeted Redis event. Each instance subscribes to user-targeted messaging events and forwards them to its local connections. Redis is best-effort after persistence/ACK.

## Idempotency
Within one PostgreSQL transaction, validate membership/reply, INSERT with ON CONFLICT DO NOTHING, then load the existing row on conflict. This makes retries return the same server message.

## Verification
Run fmt, check, clippy, tests, and diff check. Live DB/Redis/WebSocket integration remains environment-dependent.
