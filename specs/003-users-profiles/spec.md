# Feature Specification: Users & Profiles

**Feature Branch**: `003-users-profiles`
**Status**: Approved from roadmap/API/schema
**Input Sources**: `docs/22-product/roadmap.md`, `docs/12-api/api-overview.md`, `docs/10-database/schema.md`

## Scope

Implement authenticated user/profile management and user relationships required by Phase 3.

### User Stories

- Authenticated users can view and update their own profile.
- Authenticated users can view another user's public profile.
- Authenticated users can list, update, and remove their devices.
- Authenticated users can list and revoke active sessions.
- Authenticated users can add/remove platform contacts.
- Authenticated users can block/unblock users and list blocked users.
- Blocked users cannot be added as contacts and cannot be used by later communication features where block enforcement is required.

## API

- `GET /api/v1/users/me`
- `PATCH /api/v1/users/me`
- `GET /api/v1/users/{id}`
- `GET /api/v1/users/search?q={query}`
- `GET /api/v1/devices`
- `PATCH /api/v1/devices/{id}`
- `DELETE /api/v1/devices/{id}`
- `GET /api/v1/sessions`
- `DELETE /api/v1/sessions/{id}`
- `GET /api/v1/contacts`
- `POST /api/v1/contacts`
- `DELETE /api/v1/contacts/{user_id}`
- `GET /api/v1/blocks`
- `POST /api/v1/blocks`
- `DELETE /api/v1/blocks/{user_id}`

All endpoints require authentication.

## Data

Add reversible migrations for `user_profiles`, `contacts`, and `blocked_users`. Existing `users`, `devices`, and `sessions` tables are reused.

Profile fields follow the approved schema: `bio`, `phone_number`, and `avatar_url`. Actual binary avatar upload/storage is deferred to the Media/R2 phase; this phase only preserves an existing media URL when supplied by a trusted backend/media flow.

## Security

- Never expose `password_hash`.
- Users may only mutate their own profile, devices, sessions, contacts, and blocks.
- Session revocation must use the existing auth revocation path so Redis access tokens are revoked immediately.
- A user cannot contact or block themselves.
- Block/contact operations are idempotent where practical.
- Public profile responses expose only public profile fields.
