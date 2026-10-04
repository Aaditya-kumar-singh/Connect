# Project Overview — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-OVR-001`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-OVR-002, DOC-OVR-003, DOC-ARCH-001    |
| **Related ADRs**  | ADR-001 through ADR-018                    |

---

## 1. What is YBM Connect?

YBM Connect is a **production-grade, real-time communication platform** inspired by WhatsApp. It supports text messaging, media sharing, voice/video calls, presence, typing indicators, message delivery/read receipts, group conversations, and push notifications — across **web** and **Android** clients.

It is **not** a WhatsApp clone that copies proprietary internals. It is an **independently architected** system built on publicly documented distributed-systems principles, designed to be:

- A deep learning project and portfolio piece
- Architecturally sound enough to evolve toward production-scale workloads
- Simple enough for a solo developer or small team to build iteratively

## 2. Platform Support Matrix

| Platform         | Technology           | Communication Targets        |
|------------------|----------------------|------------------------------|
| Web Application  | React/Next.js + TS   | Web ↔ Web, Web ↔ Android     |
| Android App      | React Native/Expo    | Android ↔ Android, Android ↔ Web |

All communication is **cross-platform**: any client can message, call, or share media with any other client, regardless of platform.

## 3. Core Feature Set

### 3.1 Messaging
- Real-time text messaging over WebSocket
- One-to-one and group conversations
- Message delivery status (sent → delivered → read)
- Message reactions, editing, deletion
- Reply-to-message threading
- Message forwarding
- File attachments: images, videos, documents, voice messages

### 3.2 Presence & Indicators
- Online/offline presence
- Last seen timestamps
- Typing indicators (per-conversation, per-user)

### 3.3 Voice & Video Calls
- One-to-one audio calls
- One-to-one video calls
- WebRTC for media transport
- STUN/TURN for NAT traversal
- Call history

### 3.4 Authentication & Security
- Email/password registration and login
- Email OTP verification
- Argon2id password hashing
- Short-lived access tokens + refresh tokens
- Multi-device session management
- Rate limiting, abuse prevention

### 3.5 Infrastructure
- Push notifications (FCM for Android, Web Push for browsers)
- Media storage on Cloudflare R2
- Cloudflare DNS, TLS, CDN
- Structured observability (logs, metrics, traces)
- Fault-tolerant worker supervision
- Backup and disaster recovery

## 4. Technology Stack Summary

| Layer              | Technology                  | Rationale (see ADRs)           |
|--------------------|-----------------------------|-------------------------------|
| Web Frontend       | Next.js + TypeScript        | SSR, routing, React ecosystem |
| Mobile Frontend    | React Native / Expo + TS    | Cross-platform, shared logic  |
| Backend Runtime    | Rust + Tokio                | Performance, safety, async    |
| HTTP Framework     | Axum                        | Tokio-native, tower middleware|
| Primary Database   | PostgreSQL                  | ACID, relational, proven      |
| Cache / Pub-Sub    | Redis                       | Speed, pub/sub, presence      |
| Object Storage     | Cloudflare R2               | S3-compatible, no egress fees |
| Real-time Protocol | WebSocket                   | Bidirectional, low-latency    |
| Media Transport    | WebRTC                      | P2P audio/video, standard     |
| NAT Traversal      | STUN + TURN (coturn)        | Connectivity behind NATs      |
| CDN / DNS / TLS    | Cloudflare                  | Global edge, free tier        |
| Password Hashing   | Argon2id                    | Memory-hard, modern standard  |

## 5. Language Boundaries

**Principle:** One language per service, justified by a real engineering requirement.

| Boundary           | Language     | Justification                                           |
|--------------------|--------------|---------------------------------------------------------|
| Backend services   | **Rust**     | Default. Performance, memory safety, async concurrency. |
| Web frontend       | **TypeScript** | React/Next.js ecosystem requires it.                  |
| Mobile frontend    | **TypeScript** | React Native/Expo requires it.                        |
| TURN server        | **C** (coturn) | Pre-built infrastructure; no custom code needed.      |

**Erlang/OTP** is introduced in Phase 15 as a narrowly scoped reliability sidecar. It owns infrastructure-level supervision, dependency health monitoring, circuit breakers, and recovery coordination. Rust remains the primary backend runtime and retains application-level Tokio task isolation and graceful shutdown. ADR-018 records the decision; ADR-012 remains valid for Rust application-level worker isolation.

**Go** is NOT introduced. There is no service in the initial architecture where Go provides a clear advantage over Rust for the same workload.

## 6. Architectural Philosophy

The system follows a **modular monolith** approach for the backend. All Rust backend logic ships as a single binary with internal module boundaries. A service is only extracted when it meets one or more of these criteria:

1. **Independent scaling requirement** — e.g., media processing may need separate compute
2. **Independent failure domain** — e.g., TURN server must not crash with the API server
3. **Different runtime requirements** — e.g., TURN needs UDP; Axum needs TCP
4. **Different deployment lifecycle** — e.g., frontend deploys independently of backend
5. **Strong security isolation** — e.g., secrets vault runs separately
6. **Different resource profile** — e.g., video transcoding is CPU-bound; API is I/O-bound
7. **Independent ownership** — applicable for larger teams
8. **Clear operational benefit** — reduces blast radius or simplifies monitoring

This is documented in detail in [architecture-overview.md](../02-architecture/architecture-overview.md) and [ADR-011](../02-architecture/architecture-decisions/ADR-011-modular-monolith.md).

## 7. Network Boundaries

```
┌─────────────────────────────────────────────────────────┐
│                    Internet                              │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │ Web App  │  │ Android App  │  │ Other Clients    │   │
│  │ (Next.js)│  │ (Expo)       │  │ (future)         │   │
│  └────┬─────┘  └──────┬───────┘  └────────┬─────────┘   │
│       │               │                   │              │
│       └───────────────┼───────────────────┘              │
│                       │                                  │
│              HTTPS / WSS (TLS)                           │
│                       │                                  │
│       ┌───────────────▼───────────────┐                  │
│       │     Cloudflare Edge           │                  │
│       │  (DNS, TLS, CDN, WAF, R2)     │                  │
│       └───────────────┬───────────────┘                  │
│                       │                                  │
│              HTTPS / WSS                                 │
│                       │                                  │
│       ┌───────────────▼───────────────┐                  │
│       │     Rust Backend (Axum)       │                  │
│       │  ┌─────┐ ┌─────┐ ┌────────┐  │                  │
│       │  │REST │ │ WS  │ │Workers │  │                  │
│       │  └──┬──┘ └──┬──┘ └───┬────┘  │                  │
│       │     │       │        │        │                  │
│       │  ┌──▼───────▼────────▼──┐     │                  │
│       │  │  Service Layer       │     │                  │
│       │  └──┬───────────────┬───┘     │                  │
│       └─────┼───────────────┼─────────┘                  │
│             │               │                            │
│     ┌───────▼──┐     ┌──────▼──┐                         │
│     │PostgreSQL│     │  Redis  │                         │
│     └──────────┘     └─────────┘                         │
│                                                          │
│       ┌──────────────┐    ┌──────────────┐               │
│       │ TURN (coturn)│    │ Cloudflare R2│               │
│       └──────────────┘    └──────────────┘               │
└─────────────────────────────────────────────────────────┘
```

Every network boundary uses TLS. Internal services (PostgreSQL, Redis) use private networking or SSH tunnels. The TURN server uses DTLS for media relay.

## 8. Who Is This For?

| Audience               | How they use this documentation                    |
|------------------------|----------------------------------------------------|
| Solo developer         | Implementation guide, architecture reference       |
| AI coding assistant    | Architectural constraints, API contracts, rules    |
| Future contributors    | Onboarding, understanding decisions                |
| Code reviewers         | Verifying changes against architecture             |
| Ops/deployment         | Runbooks, deployment procedures, disaster recovery |

## 9. Documentation Map

See [MASTER-ENGINEERING-GUIDE.md](../MASTER-ENGINEERING-GUIDE.md) for a complete cross-referenced guide, or browse the `docs/` tree:

| Section    | Topic                    | Key Documents                          |
|------------|--------------------------|----------------------------------------|
| `00-`      | Overview                 | This document, goals, scope, glossary  |
| `01-`      | Requirements             | Business, functional, non-functional   |
| `02-`      | Architecture             | System, container, component, ADRs     |
| `03-`      | Frontend                 | Web, mobile, state, offline            |
| `04-`      | Backend                  | Rust, Axum, Tokio, modules             |
| `05-`      | Authentication           | Auth, sessions, OTP, devices           |
| `06-`      | Messaging                | Lifecycle, delivery, ordering          |
| `07-`      | Groups                   | Membership, roles, permissions         |
| `08-`      | Media                    | Upload, processing, storage            |
| `09-`      | Calls                    | WebRTC, signaling, STUN/TURN           |
| `10-`      | Database                 | Schema, indexes, migrations            |
| `11-`      | Redis                    | Cache, pub/sub, presence               |
| `12-`      | API                      | REST, WebSocket protocol               |
| `13-`      | Security                 | Threat model, encryption, rate limits  |
| `14-`      | Fault Tolerance          | Supervision, circuit breakers          |
| `15-`      | Observability            | Logs, metrics, traces, alerts          |
| `16-`      | Testing                  | Strategy, pyramid, specific tests      |
| `17-`      | DevOps                   | Local dev, CI/CD, deployment           |
| `18-`      | Cloudflare               | Workers, R2, DNS, cost model           |
| `19-`      | Performance              | Targets, benchmarks, optimization      |
| `20-`      | Development Rules        | Coding, architecture, git, AI rules    |
| `21-`      | Workflow                 | Feature dev, bug fix, release          |
| `22-`      | Product                  | Roadmap, MVP, V1, V2                   |
| `23-`      | Operations               | Runbooks, troubleshooting              |
| `24-`      | Reference                | Glossary, env vars, config, ports      |

## 10. Success Criteria

See [success-criteria.md](success-criteria.md) for detailed metrics. At a high level, the project succeeds when:

1. Two users on different platforms can exchange text messages in real-time
2. Messages survive server restarts (persistence)
3. Calls connect even behind restrictive NATs (TURN fallback)
4. The system recovers from Redis/DB failures without data loss
5. A developer can clone, set up, and run the full stack in under 15 minutes
6. The documentation answers implementation questions without requiring additional research

---

*Next: [project-goals.md](project-goals.md) · [project-scope.md](project-scope.md) · [architecture-overview.md](../02-architecture/architecture-overview.md)*
