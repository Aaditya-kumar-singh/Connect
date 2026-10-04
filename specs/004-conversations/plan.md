# Implementation Plan: Conversations

## Architecture

Create `backend/src/conversations/` with models, repository, service, and handlers.

Reuse:
- `AppState`
- `AuthenticatedSession`
- `AppError`
- Phase 3 block repository
- existing Axum router and middleware

## Persistence

1. Create `conversations`.
2. Create `conversation_members`.
3. Use a transaction for direct-conversation creation.
4. Use a PostgreSQL transaction advisory lock derived from the canonical sorted user pair to prevent concurrent duplicate creation without inventing new persistent key columns.
5. Add indexes required for user conversation listing and member lookup.

## API

Return a stable conversation representation with:
- conversation id/type/timestamps
- the other active member's public profile for direct conversations

List uses a bounded limit (1-50) and cursor based on `updated_at,id`.

## Verification

- Unit-test cursor/limit validation and pair canonicalization.
- Integration-test create, duplicate prevention, membership enforcement, blocked-user rejection, and pagination.
- Run fmt, clippy, tests, and diff audit.
- Live DB/Redis integration remains dependent on configured services.
