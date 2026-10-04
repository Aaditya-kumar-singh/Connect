# Phase 17 — Security Hardening

## Status
IMPLEMENTATION COMPLETE — DEPLOYMENT/PENTEST PENDING

## Goal
Harden the Rust/Axum API against concrete authentication, authorization, resource-exhaustion, security-configuration, and WebSocket attack paths without introducing speculative abstractions.

## Implemented
- Production configuration rejects known development secrets and localhost CORS.
- Proxy-derived client identity is opt-in through `TRUST_PROXY_HEADERS`.
- CORS methods and headers are explicitly allowlisted.
- Security response headers are applied globally.
- Global request body limit is 2 MiB; media upload has a 51 MiB route-specific limit.
- Refresh-token rotation is made atomic so concurrent reuse cannot mint multiple successor tokens.

## Remaining
- Complete authorization matrix audit for every object endpoint.
- Add security-event/audit logging for authentication and privilege-sensitive events.
- Add focused authorization and abuse regression tests.
- Review third-party HTTP integrations for explicit timeouts and SSRF-safe destination handling.
- Perform an application-level penetration test after the Cloudflare deployment using YoloPentest, PentAGI, and Strix.
