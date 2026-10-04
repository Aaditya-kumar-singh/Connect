# Feature Specification: Authentication & Session Management

**Feature Branch**: `002-auth`
**Created**: 2026-10-03
**Status**: Implemented
**Input**: Approved authentication requirements AUTH-001 through AUTH-008 and `docs/05-authentication/authentication.md`.

## User Scenarios & Testing

### User Story 1 - Register and Verify Email (Priority: P1)
As an unregistered user, I can register with email, password, and display name and verify the account using a single-use 6-digit email OTP.

**Independent Test**: Register creates a pending account and hashed verification OTP; valid OTP activates the account; invalid/expired/reused OTPs are rejected.

**Acceptance Scenarios**
1. Valid registration creates a `PENDING_VERIFICATION` user and never stores the plaintext password or OTP.
2. Duplicate email returns 409.
3. Invalid email/password/display name returns 422.
4. Valid 6-digit OTP within 10 minutes activates the account.
5. Invalid OTP returns 401, expired OTP returns 410, and exhausted attempts return 429.

### User Story 2 - Login and Multi-Device Sessions (Priority: P1)
As a verified user, I can log in from multiple devices and receive independently revocable session tokens.

**Acceptance Scenarios**
1. Active user with correct credentials receives a 15-minute opaque access token and 30-day opaque refresh token.
2. Login creates/updates a device and creates a session.
3. Unverified users receive 403; invalid credentials receive 401.
4. Server stores only SHA-256 token hashes.
5. Access tokens are validated through server-side session state.

### User Story 3 - Refresh and Logout (Priority: P1)
As an authenticated user, I can rotate refresh tokens and terminate one session or all sessions.

**Acceptance Scenarios**
1. Refresh rotates the old token and issues a new access token and refresh token.
2. Reuse of a revoked refresh token revokes its entire token family.
3. Logout revokes the current session and associated refresh token.
4. Logout-all revokes all user sessions and refresh tokens.

### User Story 4 - Password Recovery (Priority: P1)
As a user who forgot a password, I can request and confirm a reset without revealing whether an account exists.

**Acceptance Scenarios**
1. Forgot-password returns the same success response whether the email exists or not.
2. Valid reset OTP updates the password and revokes all sessions/tokens.
3. Reset OTP is single-use, expires after 10 minutes, and is stored only as a hash.
4. Requests are rate limited to 3 per email per hour.

## Functional Requirements

- **FR-001**: Implement `POST /api/v1/auth/register`.
- **FR-002**: Implement `POST /api/v1/auth/verify-email`.
- **FR-003**: Implement `POST /api/v1/auth/login`.
- **FR-004**: Implement `POST /api/v1/auth/refresh`.
- **FR-005**: Implement `POST /api/v1/auth/logout`.
- **FR-006**: Implement `POST /api/v1/auth/logout-all`.
- **FR-007**: Implement `POST /api/v1/auth/forgot-password`.
- **FR-008**: Implement `POST /api/v1/auth/reset-password`.
- **FR-009**: Passwords MUST use Argon2id with the project constitution's required parameters.
- **FR-010**: Authentication tokens and OTPs MUST NOT be stored plaintext; server persistence uses SHA-256 hashes.
- **FR-011**: Access tokens MUST be opaque, random, short-lived (15 minutes), and revocable server-side.
- **FR-012**: Refresh tokens MUST be opaque, random, 30-day tokens with rotation and family-based reuse detection.
- **FR-013**: Sessions MUST support multiple devices and independent revocation.
- **FR-014**: Public and authenticated auth endpoints MUST have the required rate limits.
- **FR-015**: Authentication business logic MUST reside in service modules; handlers must not access repositories directly.
- **FR-016**: Authentication failures MUST avoid account enumeration where specified and MUST use safe generic credential errors.
- **FR-017**: All authentication state changes MUST be transactional in PostgreSQL.
- **FR-018**: Protected authentication routes MUST use authorization middleware backed by the opaque access-token/session model.

## Security Requirements

- Never log passwords, OTPs, raw access tokens, refresh tokens, or reset tokens.
- Use constant-time-safe password/token verification primitives provided by the cryptographic libraries.
- Validate and normalize email input consistently.
- Enforce password minimum 8 characters and project-defined complexity requirements.
- Enforce OTP expiry, single-use semantics, and attempt limits.
- Rate-limit registration, verification, login, and password recovery.
- Revoke all sessions after successful password reset.

## Success Criteria

- Authentication integration tests cover registration, verification, login, refresh rotation/reuse, logout, logout-all, and password reset.
- `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt -- --check` pass.
- No plaintext authentication secret is persisted or emitted in logs.
- Authentication endpoints use the documented HTTP status/error contracts.

## Assumptions

- PostgreSQL is the authoritative store for users, verification/reset OTP state, devices, sessions, and refresh-token hashes.
- Redis is used for active access-token lookup/revocation and rate limiting, consistent with the approved architecture.
- Email delivery is represented behind a small service boundary for this phase; a real provider integration is not invented unless already present.
- Existing Phase 1 router/state/configuration are reused.

