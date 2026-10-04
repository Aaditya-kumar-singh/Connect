# Phase 17 Tasks

- [x] Audit current authentication/session controls.
- [x] Harden refresh-token rotation against concurrent reuse.
- [x] Harden proxy-header trust for rate limiting.
- [x] Tighten CORS methods and headers.
- [x] Add baseline security response headers.
- [x] Add global request-body protection with a media-upload override.
- [x] Reject insecure development secrets in production configuration.
- [x] Complete object/function authorization matrix audit for currently implemented REST operations.
- [x] Add security-event logging for authentication failures and refresh-token reuse.
- [x] Add authorization regression coverage at the service/repository boundary through existing membership/ownership checks; expand endpoint tests in the next test pass.
- [x] Review third-party HTTP integrations and SSRF controls; FCM has bounded 10s HTTP timeouts and Web Push endpoints are HTTPS-only, provider-host allowlisted, and reject IP literals.
- [ ] Run application penetration testing after Cloudflare deployment; execute YoloPentest, PentAGI, and Strix against the authorized deployment.
