# Tasks: Authentication & Session Management

**Input**: `spec.md` and `plan.md`
**Prerequisites**: Phase 1 infrastructure complete.

## Phase 1: Data & Authentication Foundation

- [x] T001 Add reversible PostgreSQL migrations for users, email verification/OTP state, devices, sessions, and refresh tokens using the approved schema.
- [x] T002 Add auth domain models and typed validation errors.
- [x] T003 Implement Argon2id password hashing/verification with constitution-required parameters.
- [x] T004 Implement secure opaque token generation and SHA-256 hashing.
- [x] T005 Implement auth repositories for user, OTP, device, session, and refresh-token persistence.

## Phase 2: Registration & Verification

- [x] T006 [US1] Implement registration validation and service.
- [x] T007 [US1] Implement email OTP generation, hashing, expiry, attempt limit, and single-use verification.
- [x] T008 [US1] Implement `POST /api/v1/auth/register`.
- [x] T009 [US1] Implement `POST /api/v1/auth/verify-email`.
- [x] T010 [US1] Add integration tests for registration and verification.

## Phase 3: Login & Sessions

- [x] T011 [US2] Implement login credential verification and safe failure handling.
- [x] T012 [US2] Implement device upsert and session creation.
- [x] T013 [US2] Implement access-token storage/lookup and authentication extractor/middleware.
- [x] T014 [US2] Implement `POST /api/v1/auth/login`.
- [x] T015 [US2] Add login, multi-device, and protected-route integration tests.

## Phase 4: Refresh & Logout

- [x] T016 [US3] Implement refresh-token rotation and family reuse detection.
- [x] T017 [US3] Implement `POST /api/v1/auth/refresh`.
- [x] T018 [US3] Implement logout and logout-all services and handlers.
- [x] T019 [US3] Add refresh/reuse/logout integration tests.

## Phase 5: Password Recovery

- [x] T020 [US4] Implement non-enumerating forgot-password service.
- [x] T021 [US4] Implement reset OTP verification and password replacement.
- [x] T022 [US4] Implement `POST /api/v1/auth/forgot-password` and `POST /api/v1/auth/reset-password`.
- [x] T023 [US4] Add password-reset integration tests including session revocation.

## Phase 6: Rate Limiting & Verification

- [x] T024 Implement the auth endpoint rate limits using Redis without duplicating a general rate-limit abstraction if one already exists.
- [x] T025 Add HTTP contract tests for status codes and generic error responses.
- [x] T026 Run `cargo fmt -- --check`.
- [x] T027 Run `cargo clippy -- -D warnings`.
- [ ] T028 Run `cargo test` with PostgreSQL and Redis integration services. Blocked on this workstation because Docker is not installed; the ignored integration test compiles and will run when TEST_DATABASE_URL and TEST_REDIS_URL are provided.
- [x] T029 Audit `git diff` against spec, plan, tasks, AGENTS.md, GEMINI.md, and Ponytail rules; remove unrelated changes.

