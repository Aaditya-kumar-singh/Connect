# Tasks: Users & Profiles

## Data

- [x] T001 Add reversible `user_profiles` migration.
- [x] T002 Add reversible `contacts` and `blocked_users` migrations.

## Users

- [x] T003 Implement profile models/repository.
- [x] T004 Implement `GET/PATCH /users/me`.
- [x] T005 Implement `GET /users/{id}`.
- [x] T006 Implement bounded `GET /users/search`.

## Devices & Sessions

- [x] T007 Implement device listing/update/removal.
- [x] T008 Implement active-session listing and individual session revocation.

## Contacts & Blocks

- [x] T009 Implement contact list/add/remove.
- [x] T010 Implement block list/add/remove and block/contact enforcement.

## Verification

- [x] T011 Add integration tests for profile ownership, device/session ownership, contacts, and blocks.
- [x] T012 Run `cargo fmt -- --check`.
- [x] T013 Run `cargo clippy -- -D warnings`.
- [ ] T014 Run `cargo test` with PostgreSQL and Redis services. Blocked on this workstation because Docker is not installed; the integration test is compiled and ignored until TEST_DATABASE_URL and TEST_REDIS_URL are supplied.
- [x] T015 Audit diff against spec, plan, AGENTS.md, GEMINI.md, and Ponytail rules.

