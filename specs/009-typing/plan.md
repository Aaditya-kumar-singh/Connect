# Phase 9 Plan — Typing Indicators

## Architecture
Reuse the existing WebSocket dispatcher, \`AuthenticatedSession\`, conversation membership table, \`ConnectionManager\`, Redis rate limiter, and Redis user Pub/Sub channel.

## Components
- \`backend/src/typing.rs\`: validation, TTL state, payload/frame construction, and broadcast.
- \`backend/src/websocket/connection.rs\`: dispatch and rate-limit integration.
- \`backend/src/lib.rs\`: expose typing module.
- \`backend/tests/typing.rs\`: protocol and TTL contract tests.

## Data
No PostgreSQL migration. Redis owns ephemeral typing state:
\`typing:{conversation_id}:{user_id}\` → short-lived marker, TTL 5 seconds.

## Delivery
Local connections are sent through \`ConnectionManager\`. Each recipient also receives a best-effort Redis Pub/Sub publish through the existing \`messaging:user:{user_id}\` channel, allowing another application instance to deliver the frame.

## Verification
Run formatting, compile/check, Clippy with warnings denied, tests, and \`git diff --check\`. Live Redis/PostgreSQL/WebSocket integration remains separate and must not be claimed unless those services are available and exercised.
