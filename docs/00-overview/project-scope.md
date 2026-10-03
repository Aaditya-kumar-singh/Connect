# Project Scope — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-OVR-003`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-OVR-001, DOC-OVR-002, DOC-OVR-004     |
| **Related ADRs**  | ADR-011, ADR-013                           |

---

## In Scope

### Platform
| Component         | Scope                                       |
|-------------------|---------------------------------------------|
| Web application   | Full-featured client (Next.js + TypeScript)  |
| Android app       | Full-featured client (Expo + TypeScript)     |
| Backend API       | Rust + Axum monolith                         |
| Database          | PostgreSQL                                   |
| Cache             | Redis                                        |
| Object storage    | Cloudflare R2                                |
| TURN server       | coturn (self-hosted or managed)              |
| CDN/DNS/TLS       | Cloudflare                                   |

### Features (MVP through V1)

| Category          | Features                                              |
|-------------------|-------------------------------------------------------|
| Authentication    | Email/password, email OTP, sessions, multi-device     |
| Messaging         | Send, receive, deliver, read, edit, delete, reply, forward |
| Media             | Images, videos, documents, voice messages             |
| Presence          | Online/offline, last seen                             |
| Indicators        | Typing start/stop per conversation                    |
| Groups            | Create, membership, roles, permissions, group messaging |
| Calls             | 1-to-1 audio, 1-to-1 video, call history             |
| Notifications     | Push (FCM, Web Push)                                  |
| Security          | Rate limiting, input validation, CSRF, XSS prevention |
| Observability     | Structured logging, metrics, health checks            |
| DevOps            | Docker Compose local dev, CI/CD, deployment scripts   |

### Quality Attributes
- Sub-second message delivery (p95 < 500ms on same region)
- 99.5% uptime target for learning/portfolio deployment
- Graceful degradation on component failure
- Horizontal scalability path without architecture rewrite

## Out of Scope

See [non-goals.md](non-goals.md) for rationale on each exclusion.

| Exclusion                  | Reason                                        |
|----------------------------|-----------------------------------------------|
| End-to-end encryption      | Significant complexity; deferred to V2        |
| Group calls                | 1-to-1 calls first; group SFU deferred to V2 |
| Status/stories             | Not core messaging; deferred                  |
| Desktop native app         | Web app covers desktop use cases initially    |
| iOS app                    | Android first; iOS after Android stabilizes   |
| Self-destructing messages  | Deferred to V2                                |
| Payments/commerce          | Not relevant to the platform's purpose        |
| Admin dashboard            | CLI/API tools initially; dashboard deferred   |
| Message search (full-text) | Basic search in V1; advanced in V2            |
| AI features                | Out of scope                                  |

## Scope Boundaries

### What the Backend Handles
- All business logic (auth, messaging, groups, calls, media)
- All persistence (PostgreSQL writes/reads)
- All real-time routing (WebSocket → Redis pub/sub → WebSocket)
- All media processing (thumbnails, validation)
- Call signaling (SDP, ICE candidate relay)

### What the Backend Does NOT Handle
- Media transport during calls (WebRTC peer-to-peer or via TURN)
- Client-side UI rendering
- Client-side state management
- Push notification delivery (delegates to FCM/Web Push services)

### What Cloudflare Handles
- DNS resolution
- TLS termination at edge
- Static asset CDN for frontend
- Object storage for uploaded media (R2)
- DDoS protection

### What Cloudflare Does NOT Handle
- Long-running WebSocket connections (Axum handles these)
- Database operations
- Business logic
- Call signaling

---

*Next: [non-goals.md](non-goals.md) · [terminology.md](terminology.md)*
