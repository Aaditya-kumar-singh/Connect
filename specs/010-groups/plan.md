# Phase 10 Plan — Groups

1. Add groups and group_members migrations.
2. Allow nullable message sender IDs for system messages.
3. Implement group repository/service/handlers.
4. Reuse existing conversation_members for message authorization and group fan-out.
5. Add all documented REST routes.
6. Add system messages for group state changes.
7. Add unit and live-integration placeholders.
8. Run fmt/check/clippy/test/diff verification.

The live integration test remains separate until PostgreSQL, Redis, and authenticated clients are available.
