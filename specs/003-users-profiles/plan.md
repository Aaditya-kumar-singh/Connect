# Implementation Plan: Users & Profiles

## Architecture

Add `backend/src/users/` and `backend/src/contacts/` modules.

Reuse:
- `AppState`
- existing `AuthenticatedSession` extractor
- existing `AppError`
- existing session revocation service
- existing PostgreSQL/Redis infrastructure

Avoid new dependencies.

## Data Layer

1. Create `user_profiles` one-to-one with users.
2. Create `contacts` with composite primary key.
3. Create `blocked_users` with composite primary key.
4. Add indexes/constraints matching the approved schema.

## Services

### Users
- current profile read/update
- public profile read
- bounded user search

### Devices/Sessions
- list owned devices
- update own device push token/provider
- remove own device and revoke its sessions
- list own active sessions
- revoke one own session

### Contacts/Blocks
- list contacts/blocks
- add/remove contact
- block/unblock user
- prevent self-relations
- prevent blocked relationship from being added as a contact

## Verification

- Unit-test validation and response shaping.
- Integration-test authenticated ownership and relationship rules.
- Run fmt, clippy, tests, and diff audit.
- Live PostgreSQL/Redis integration remains dependent on configured services.
