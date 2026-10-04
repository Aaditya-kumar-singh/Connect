# Implementation Plan: Authentication & Session Management

**Branch**: `002-auth` | **Date**: 2026-10-03 | **Status**: Implemented | **Spec**: [spec.md](spec.md)

## Summary

Implement the approved authentication contract as a modular Rust/Axum feature without introducing a separate service.

Architecture:
Transport handlers → Auth service → repositories → PostgreSQL/Redis.

The feature will reuse the Phase 1 `AppState`, router, configuration, error handling, tracing, and middleware.

## Constitution Check

- [x] Modular monolith and strict layering.
- [x] PostgreSQL remains authoritative for durable account/session records.
- [x] Redis remains limited to ephemeral access-token state and rate limiting.
- [x] Argon2id and hashed-token requirements are preserved.
- [x] Tests cover cryptographic and state-transition paths.

## Planned Structure

```text
backend/src/
  auth/
    mod.rs
    handlers.rs
    service.rs
    repository.rs
    models.rs
    tokens.rs
    validation.rs
    email.rs
  routes/mod.rs
  app_state.rs
```

Database migrations will be added only for tables required by AUTH-001–008 and will follow the already-approved database schema documentation.

## Design Decisions

1. Keep access and refresh tokens opaque; do not introduce JWT.
2. Generate tokens from cryptographically secure randomness.
3. Store SHA-256 hashes, never raw token material.
4. Keep refresh-token family identifiers to detect reuse.
5. Keep password hashing in the service/security boundary and use the constitution-required Argon2id parameters.
6. Keep email delivery behind an interface so tests can use a deterministic fake.
7. Make registration, verification, login/session creation, refresh rotation, logout, and reset transactional where durable state changes occur.
8. Do not implement unrelated user profiles, messaging, WebSocket, push notification, or frontend work in this phase.

## Verification

- Unit tests for validation, hashing, token generation/hash, OTP expiry/attempt logic, and refresh-family rules.
- Integration tests against real PostgreSQL/Redis containers for repository/service contracts.
- HTTP tests for all eight endpoints and their documented status codes.
- Run format, clippy, and full test suite before completion.

