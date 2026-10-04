# MASTER ENGINEERING GUIDE — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-MASTER-001`                           |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |

---

## 1. Project Vision

YBM Connect is a production-grade, real-time communication platform supporting text messaging, media sharing, voice/video calls, presence, and push notifications across web and Android clients. It is designed as a learning/portfolio project with production-quality architecture.

**Build fast. Keep architecture clean. Make failures recoverable. Make the system easy to debug. Make future scaling possible. Avoid unnecessary complexity.**

## 2. Architecture

**Style:** Modular Monolith (single Rust binary with internal module boundaries)
**Extraction Rule:** Only split a module into a separate service when it has an independent scaling requirement, failure domain, runtime requirement, deployment lifecycle, security isolation need, resource profile, ownership boundary, or clear operational benefit.

### Deployment Units
1. **Rust Backend** — Axum (REST + WebSocket + background workers)
2. **Erlang/OTP Reliability Service** — OTP sidecar (supervision, circuit breakers, dependency monitoring)
3. **Web Frontend** — Next.js + TypeScript
4. **Mobile App** — React Native / Expo + TypeScript
5. **TURN Server** — coturn (pre-built, separate because of UDP/different protocol)

### Layered Architecture (Backend)
```
Transport (Axum handlers) → Service (business logic) → Repository (data access) → Infrastructure (DB/Redis/R2)
```
Domain types are shared across all layers. Business logic ONLY lives in the Service layer.

### Technology Stack
| Component | Technology | ADR |
|-----------|-----------|-----|
| Backend | Rust + Tokio + Axum | ADR-001, 002, 003 |
| Reliability | Erlang/OTP 27 + rebar3 | ADR-018 |
| Database | PostgreSQL | ADR-004 |
| Cache/PubSub | Redis | ADR-005 |
| Real-time | WebSocket | ADR-006 |
| Calls | WebRTC + STUN/TURN | ADR-007, 008 |
| CDN/DNS/TLS | Cloudflare | ADR-009 |
| Object Storage | Cloudflare R2 | ADR-010 |
| Password Hashing | Argon2id | ADR-014 |
| Web Frontend | Next.js + TypeScript | — |
| Mobile Frontend | Expo + TypeScript | — |

### Language Boundaries
- **Rust:** All backend application services. Primary backend language.
- **Erlang/OTP:** Reliability sidecar only (supervision, circuit breakers, dependency monitoring). Must not contain business logic.
- **TypeScript:** All frontend code (web + mobile).
- **C (coturn):** Pre-built TURN server. No custom code.

## 3. Database

**22 tables** in PostgreSQL. Key tables: `users`, `conversations`, `messages`, `message_receipts`, `sessions`, `devices`, `groups`, `calls`, `media_objects`.

**Critical design decisions:**
- `client_message_id` with UNIQUE constraint for idempotency (ADR-015)
- Server timestamp for message ordering (ADR-016)
- Soft deletes for messages (preserves referential integrity)
- Append-only audit_logs (no UPDATE/DELETE permissions)
- Token hashes stored, never plaintext tokens

Full schema: [docs/10-database/schema.md](docs/10-database/schema.md)

## 4. Messaging

**State machine:** CREATED → SENT → VALIDATED → PERSISTED → ACKNOWLEDGED → DELIVERED → READ

**Critical rule:** ACK is sent to sender ONLY AFTER PostgreSQL transaction commits. This guarantees message durability.

**Idempotency:** Client generates UUID v7 as `client_message_id`. Server uses `INSERT ... ON CONFLICT DO NOTHING`. Duplicate sends return the existing message.

**Cross-instance routing:** Redis pub/sub. Each instance subscribes to channels for conversations with locally connected members.

**Offline delivery:** Messages are always in PostgreSQL. On reconnect, client sends `conversation.sync` with last known message ID. Server returns all newer messages.

Full spec: [docs/06-messaging/messaging-architecture.md](docs/06-messaging/messaging-architecture.md)

## 5. WebSocket Protocol

**Connection:** `GET /ws/connect?token={access_token}&device_id={device_id}`
**Heartbeat:** Client sends `ping` every 30s. Server drops connection after 60s silence.
**Reconnection:** Exponential backoff (1s → 30s max) with jitter.

**Key events:**
| Event | Direction | Purpose |
|-------|-----------|---------|
| `message.send` | C→S | Send message |
| `message.ack` | S→C | Acknowledge persistence |
| `message.new` | S→C | Deliver new message |
| `message.delivered` | C→S | Delivery receipt |
| `message.read` | C→S | Read receipt |
| `typing.start/stop` | Both | Typing indicators |
| `presence.update` | S→C | Online/offline status |
| `call.offer/answer/ice_candidate` | Both | Call signaling |

Full spec: [docs/12-api/websocket-protocol.md](docs/12-api/websocket-protocol.md)

## 6. WebRTC

**Signaling:** Via WebSocket through Rust backend (SDP offers/answers, ICE candidates)
**Media:** Direct P2P between clients, or relayed via TURN server
**State machine:** IDLE → CALLING → RINGING → CONNECTING → CONNECTED → ENDED (with RECONNECTING, REJECTED, MISSED, FAILED branches)

The Rust backend NEVER touches media data. It only relays signaling messages.

Full spec: [docs/09-calls/webrtc.md](docs/09-calls/webrtc.md)

## 7. Redis

**Purpose:** Session cache, presence, typing indicators, cross-instance message routing (pub/sub), rate limiting, connection registry.
**NOT a data store.** All persistent data lives in PostgreSQL.
**Failure impact:** Real-time features degrade. Messages still persist and are delivered on sync.
**Every key has a TTL.** No unbounded memory growth.

Full spec: [docs/11-redis/redis-architecture.md](docs/11-redis/redis-architecture.md)

## 8. Security

**Authentication:** Argon2id password hashing, short-lived opaque access tokens (15 min), refresh token rotation with reuse detection, email OTP verification.
**Authorization:** Every endpoint verifies the user is authorized for the requested resource.
**Input validation:** Server-side validation on all user input. Magic byte validation for file uploads.
**Rate limiting:** Multi-layer (Cloudflare edge + application-level per-user + business logic limits).
**18 documented threats** with prevention and detection strategies.

Full spec: [docs/13-security/security-architecture.md](docs/13-security/security-architecture.md)

## 9. Fault Tolerance

**Dual-runtime architecture (ADR-012 + ADR-018):**
- **Rust / Tokio (application-level):** Tokio task panic isolation, mpsc channel-based worker communication, CancellationToken graceful shutdown, manual worker restart.
- **Erlang/OTP (infrastructure-level):** OTP supervision trees with restart strategies and intensity limits, circuit breakers for external dependencies (PostgreSQL, Redis, R2), dependency health monitors, recovery coordinator.

**Supervision tree (Erlang/OTP):** Root supervisor → health aggregator, dependency monitors, circuit breakers, recovery coordinator. Each supervised with `one_for_one` strategy.
**Circuit breakers:** CLOSED → OPEN (on failure threshold) → HALF_OPEN (on cooldown) → CLOSED (on successful probe). Per-dependency configuration.
**Communication:** Rust ↔ Erlang via Redis Pub/Sub (`ybm:reliability:events`, `ybm:reliability:status`).
**Key principle:** Erlang is advisory, not blocking. Rust operates independently if Erlang is unavailable.
**Graceful shutdown:** SIGTERM → stop accepting connections → drain workers → close WebSocket connections → Erlang stops monitoring → close DB/Redis → exit.

Full spec: [docs/14-fault-tolerance/fault-tolerance.md](docs/14-fault-tolerance/fault-tolerance.md)

## 10. Testing

**Pyramid:** 60% unit (no I/O, fast) → 30% integration (real DB/Redis) → 5% E2E → 5% load.
**Messaging tests:** Duplicate send, reconnect, offline recipient, server restart, DB failure, Redis failure, out-of-order delivery.
**Call tests:** Accept, reject, timeout, disconnect, ICE failure, TURN fallback.

Full spec: [docs/16-testing/testing-strategy.md](docs/16-testing/testing-strategy.md)

## 11. Deployment

**Local:** Docker Compose (PostgreSQL + Redis + MinIO) + `just dev`
**Production backend:** Single binary deployed via container or systemd
**Production frontend:** Static deploy to Cloudflare Pages or similar
**TURN:** coturn deployed separately

The backend is a traditional long-running server. It does NOT run inside Cloudflare Workers (Axum needs persistent WebSocket connections which Workers don't support well).

Full spec: [docs/17-devops/local-development.md](docs/17-devops/local-development.md)

## 12. Observability

**Logging:** JSON-structured via `tracing`. Required fields: timestamp, level, target, request_id. Never log secrets.
**Metrics:** Prometheus-compatible at `/metrics`. HTTP latency, WebSocket connections, message throughput, DB pool usage.
**Health checks:** `/health` (liveness), `/ready` (readiness with DB/Redis/R2 checks).
**Alerts:** High error rate, high latency, connection spikes, DB exhaustion, Redis unavailable.

Full spec: [docs/15-observability/logging.md](docs/15-observability/logging.md)

## 13. Development Rules

**30 engineering rules** covering architecture, code quality, operations, performance, and concurrency.
**30 AI development rules** preventing common AI-generated code problems.
**AI workflow:** READ → UNDERSTAND → PLAN → MODIFY → TEST → REVIEW → DOCUMENT

Full spec: [docs/20-development-rules/coding-standards.md](docs/20-development-rules/coding-standards.md)

## 14. Git Workflow

**Branches:** `main` (production) → `develop` (integration) → `feature/*`, `fix/*`, `hotfix/*`
**Commits:** `{type}({scope}): {description}` (feat, fix, refactor, docs, test, etc.)
**PR template:** Problem, Solution, Files, Architecture impact, DB changes, API changes, Security impact, Tests, Rollback plan.

Full spec: [docs/20-development-rules/git-rules.md](docs/20-development-rules/git-rules.md)

## 15. Roadmap

**21 phases** from project foundation through production hardening.
**Estimated timeline:** 71-107 days for a solo developer.
**Key milestones:**
- Phase 2: Authentication ✓
- Phase 6: Text messaging ✓
- Phase 13-14: Audio/video calls ✓
- Phase 20: Production hardening ✓

Full spec: [docs/22-product/roadmap.md](docs/22-product/roadmap.md)

## 16. Definition of Done

A feature is complete only when:
- [ ] Requirements documented
- [ ] Architecture reviewed
- [ ] Code implemented
- [ ] Tests added (unit + integration)
- [ ] Error handling implemented
- [ ] Security reviewed (auth, authz, validation)
- [ ] Logging implemented (structured, with request_id)
- [ ] Metrics added where appropriate
- [ ] API documentation updated
- [ ] Database migration added if needed
- [ ] Diagrams updated if architecture changed
- [ ] Local test successful
- [ ] Integration test successful
- [ ] Code review completed
- [ ] Deployment tested
- [ ] Rollback understood

## 17. Quick Reference

### Key Commands
```bash
just dev          # Start everything
just test         # Run all tests
just db-migrate   # Apply migrations
just db-reset     # Reset database
just lint         # Lint code
just fmt          # Format code
```

### Key URLs (Local Development)
| Service | URL |
|---------|-----|
| Backend API | http://localhost:8080/api/v1/ |
| WebSocket | ws://localhost:8080/ws/connect |
| Health check | http://localhost:8080/health |
| Metrics | http://localhost:8080/metrics |
| Web frontend | http://localhost:3000 |
| MinIO console | http://localhost:9001 |

### Key Files
| File | Purpose |
|------|---------|
| `backend/src/main.rs` | Backend entry point |
| `backend/src/config.rs` | Configuration loading |
| `backend/migrations/` | Database migrations |
| `.env.example` | Environment template |
| `docker-compose.yml` | Local infrastructure |
| `justfile` | Development commands |

---

*This document is the single-page reference for the entire YBM Connect project. For detailed specifications, follow the links to the full documentation tree.*
