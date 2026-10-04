# Phase 17 Security Hardening Plan

1. Audit authentication/session lifecycle.
2. Audit object and function authorization.
3. Audit resource limits and multipart/WebSocket abuse controls.
4. Harden CORS, security headers, proxy trust, and production configuration.
5. Review external integrations and secret handling.
6. Add regression tests for concrete findings.
7. Run formatting, compilation, clippy, focused tests, and diff audit.

## Boundary
Rust remains the application/business runtime. No new service or dependency is introduced unless a verified security gap requires it.
