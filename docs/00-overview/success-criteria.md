# Success Criteria — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-OVR-007`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-OVR-001, DOC-OVR-002, DOC-PERF-001    |

---

## Functional Success Criteria

| ID       | Criterion                                                              | Measurement                           |
|----------|------------------------------------------------------------------------|---------------------------------------|
| SC-F-01  | Two users can register and authenticate                                 | Manual test, integration test         |
| SC-F-02  | Two users can exchange text messages in real-time                       | WebSocket integration test            |
| SC-F-03  | Messages persist across server restarts                                 | Restart server, verify messages exist |
| SC-F-04  | Messages are delivered to offline users on reconnection                 | Disconnect, send, reconnect, verify   |
| SC-F-05  | Delivery and read receipts update correctly                             | Multi-client test                     |
| SC-F-06  | Group messaging works with 3+ participants                              | Group integration test                |
| SC-F-07  | Media (images, documents) upload and display correctly                  | Upload test, download test            |
| SC-F-08  | Voice messages record and play back                                     | Record, send, play on recipient       |
| SC-F-09  | 1-to-1 audio call connects and transmits audio                         | WebRTC call test                      |
| SC-F-10  | 1-to-1 video call connects and transmits video                         | WebRTC call test                      |
| SC-F-11  | Calls work behind NAT using TURN                                       | Test with TURN-only ICE config        |
| SC-F-12  | Presence shows online/offline/last seen                                 | Multi-client observation              |
| SC-F-13  | Typing indicators appear and disappear                                  | Multi-client test                     |
| SC-F-14  | Push notifications arrive when app is backgrounded                      | Background the app, send message      |
| SC-F-15  | Multi-device: messages appear on all user devices                       | Login on 2 devices, send, verify both |
| SC-F-16  | Web ↔ Android cross-platform messaging works                           | Cross-platform test                   |

## Performance Success Criteria

| ID       | Metric                        | Target        | Maximum Acceptable | Measurement Method         |
|----------|-------------------------------|---------------|--------------------|----------------------------|
| SC-P-01  | Message delivery latency (p95)| < 300ms       | < 1000ms           | Timestamp diff sender→recipient |
| SC-P-02  | REST API latency (p95)        | < 200ms       | < 500ms            | Server-side histogram      |
| SC-P-03  | WebSocket connect time        | < 500ms       | < 2000ms           | Client-side measurement    |
| SC-P-04  | Database query latency (p95)  | < 50ms        | < 200ms            | Query timing middleware    |
| SC-P-05  | Media upload (1MB image)      | < 3s          | < 10s              | Client-side measurement    |
| SC-P-06  | Call setup time               | < 3s          | < 8s               | Offer-to-connected delta   |
| SC-P-07  | Concurrent WebSocket conns    | 1,000         | —                  | Load test                  |
| SC-P-08  | Messages per second (single instance) | 500    | —                  | Load test                  |

## Reliability Success Criteria

| ID       | Criterion                                                    | Measurement                        |
|----------|--------------------------------------------------------------|------------------------------------|
| SC-R-01  | No message loss during Redis restart                          | Restart Redis, verify messages     |
| SC-R-02  | System recovers from database connection pool exhaustion      | Induce exhaustion, verify recovery |
| SC-R-03  | Workers restart after panic                                   | Induce panic, verify restart       |
| SC-R-04  | Graceful shutdown completes in-flight requests                | SIGTERM, verify no dropped requests|
| SC-R-05  | Client reconnects automatically after network interruption    | Disconnect network, reconnect      |

## Developer Experience Success Criteria

| ID       | Criterion                                                    | Measurement                        |
|----------|--------------------------------------------------------------|------------------------------------|
| SC-D-01  | Clone-to-running in < 15 minutes                             | Timed manual test                  |
| SC-D-02  | `docker compose up` starts all dependencies                   | Single command test                |
| SC-D-03  | All tests pass in CI within 10 minutes                        | CI pipeline timing                 |
| SC-D-04  | A developer can find architecture answers in docs             | Qualitative review                 |
| SC-D-05  | AI assistant can implement features using docs alone          | Assisted development test          |

## Security Success Criteria

| ID       | Criterion                                                    | Measurement                        |
|----------|--------------------------------------------------------------|------------------------------------|
| SC-S-01  | Passwords stored with Argon2id, never plaintext               | Database inspection                |
| SC-S-02  | SQL injection attempts return errors, not data                | Penetration test                   |
| SC-S-03  | Rate limiting blocks brute-force login attempts               | Automated login attack test        |
| SC-S-04  | Expired tokens are rejected                                   | Token expiry test                  |
| SC-S-05  | Users cannot access other users' conversations                | IDOR test                          |
| SC-S-06  | File uploads are validated (type, size, content)              | Malicious file upload test         |

---

*Next: [business-requirements.md](../01-requirements/business-requirements.md)*
