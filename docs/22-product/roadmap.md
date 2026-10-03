# Implementation Roadmap — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-PROD-001`                             |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-OVR-001, DOC-REQ-007                  |

---

## Phase 0: Project Foundation

**Goal:** Establish project repository, documentation, and tooling.

| Item | Details |
|------|---------|
| **Features** | Repository setup, documentation structure, architecture decisions |
| **Files/Modules** | `docs/` tree, `.env.example`, `justfile`, `docker-compose.yml`, `.gitignore`, `README.md` |
| **Database** | None |
| **APIs** | None |
| **Tests** | None |
| **Definition of Done** | Repo cloneable, `docker compose up` works, docs readable |
| **Risks** | None — foundational setup |
| **Dependencies** | Git, Docker, Rust toolchain |
| **Estimated Complexity** | Low (1-2 days) |
| **Rollback** | N/A |

---

## Phase 1: Repository & Dev Infrastructure

**Goal:** Backend project skeleton, CI/CD pipeline, database setup.

| Item | Details |
|------|---------|
| **Features** | Cargo workspace, Axum hello-world, PostgreSQL connection, Redis connection, CI pipeline |
| **Architecture Changes** | Initial backend crate structure (domain, config, infrastructure) |
| **Files/Modules** | `backend/Cargo.toml`, `src/main.rs`, `src/config.rs`, `src/infrastructure/{database,redis}.rs` |
| **Database** | PostgreSQL connection pool (sqlx), empty schema |
| **APIs** | `GET /health`, `GET /ready` |
| **Tests** | Health check test, DB connection test |
| **Definition of Done** | Backend starts, connects to PostgreSQL and Redis, health endpoint returns 200, CI passes |
| **Risks** | Toolchain setup issues on Windows |
| **Dependencies** | Phase 0 |
| **Estimated Complexity** | Low (2-3 days) |
| **Rollback** | Delete backend directory |

---

## Phase 2: Authentication

**Goal:** Complete email/password auth with OTP verification, sessions, and tokens.

| Item | Details |
|------|---------|
| **Features** | Register, email OTP, login, logout, token refresh, password reset, session management |
| **Architecture Changes** | Auth module (handlers, service, repository), middleware (auth extraction), email adapter |
| **Files/Modules** | `src/auth/`, `src/middleware/auth.rs`, `src/infrastructure/email.rs` |
| **Database** | Migration: `users`, `user_profiles`, `email_verifications`, `otp_requests`, `sessions`, `devices`, `refresh_tokens` |
| **APIs** | `POST /auth/register`, `/auth/verify-email`, `/auth/login`, `/auth/refresh`, `/auth/logout`, `/auth/logout-all`, `/auth/forgot-password`, `/auth/reset-password` |
| **Tests** | Unit: password validation, OTP generation. Integration: full auth flow, rate limiting, token rotation, reuse detection |
| **Definition of Done** | User can register, verify, login, refresh, logout. Tokens expire correctly. Rate limiting works. |
| **Risks** | Email delivery in development (mitigated by console email provider) |
| **Dependencies** | Phase 1 |
| **Estimated Complexity** | Medium (5-7 days) |
| **Rollback** | Revert migration, remove auth module |

---

## Phase 3: Users & Profiles

**Goal:** User profile management, device management, contacts.

| Item | Details |
|------|---------|
| **Features** | View/update profile, avatar upload, device list, session list, add/remove contacts, block users |
| **Files/Modules** | `src/users/`, `src/contacts/` |
| **Database** | Migration: `blocked_users`, `contacts` (users, user_profiles already exist) |
| **APIs** | `GET/PATCH /users/me`, `GET /users/{id}`, `GET /devices`, `DELETE /devices/{id}`, `POST/DELETE /contacts`, `POST/DELETE /blocks` |
| **Tests** | Profile CRUD, contact management, block enforcement |
| **Definition of Done** | User can view and update profile, manage devices, manage contacts |
| **Dependencies** | Phase 2 |
| **Estimated Complexity** | Medium (3-5 days) |

---

## Phase 4: Conversations

**Goal:** Create and manage 1-to-1 conversations.

| Item | Details |
|------|---------|
| **Features** | Create direct conversation, list conversations, conversation detail |
| **Files/Modules** | `src/conversations/` |
| **Database** | Migration: `conversations`, `conversation_members` |
| **APIs** | `POST /conversations`, `GET /conversations`, `GET /conversations/{id}` |
| **Tests** | Create conversation, list, duplicate prevention (same pair) |
| **Definition of Done** | Two users can create a conversation, list their conversations, view conversation details |
| **Dependencies** | Phase 3 |
| **Estimated Complexity** | Low-Medium (2-3 days) |

---

## Phase 5: Real-Time WebSocket Infrastructure

**Goal:** WebSocket connection, authentication, heartbeat, reconnection support.

| Item | Details |
|------|---------|
| **Features** | WebSocket upgrade, token auth, heartbeat/pong, connection registry, graceful disconnect |
| **Architecture Changes** | WebSocket module, connection manager, Redis pub/sub subscriber |
| **Files/Modules** | `src/websocket/`, `src/infrastructure/redis.rs` (pub/sub) |
| **Database** | None (uses existing sessions) |
| **APIs** | `GET /ws/connect` (WebSocket upgrade) |
| **Tests** | Connect, auth, heartbeat timeout, reconnect, multiple connections per user |
| **Definition of Done** | Client connects via WebSocket, authenticates, heartbeat works, connection tracked per device |
| **Dependencies** | Phase 2 (auth), Phase 4 (conversations) |
| **Estimated Complexity** | High (5-7 days) |

---

## Phase 6: Text Messaging

**Goal:** Send and receive text messages in real-time.

| Item | Details |
|------|---------|
| **Features** | Send message, receive in real-time, message ACK, idempotency, edit, delete, reply, forward, reactions |
| **Architecture Changes** | Messaging module, Redis pub/sub integration, WebSocket event handlers |
| **Files/Modules** | `src/messaging/` |
| **Database** | Migration: `messages`, `message_edits`, `message_reactions`, `message_attachments` |
| **WebSocket Events** | `message.send`, `message.ack`, `message.new`, `message.edit`, `message.delete`, `message.react` |
| **Tests** | All messaging test scenarios (see testing strategy) |
| **Definition of Done** | Two users can exchange messages in real-time, messages persist, idempotency works, edit/delete/reply/react work |
| **Dependencies** | Phase 5 |
| **Estimated Complexity** | High (7-10 days) |

---

## Phase 7: Message Receipts

**Goal:** Delivery and read receipts.

| Item | Details |
|------|---------|
| **Database** | Migration: `message_receipts` |
| **WebSocket Events** | `message.delivered`, `message.read`, `message.status` |
| **Definition of Done** | Sender sees delivered/read status, receipts persist |
| **Dependencies** | Phase 6 |
| **Estimated Complexity** | Medium (3-4 days) |

---

## Phase 8: Presence

**Goal:** Online/offline status, last seen.

| Item | Details |
|------|---------|
| **Features** | Online/offline tracking, last seen, presence broadcast |
| **Files/Modules** | `src/presence/` |
| **Redis** | `presence:{user_id}` with TTL |
| **WebSocket Events** | `presence.update` |
| **Definition of Done** | Users see who is online, last seen updates on disconnect |
| **Dependencies** | Phase 5 |
| **Estimated Complexity** | Medium (2-3 days) |

---

## Phase 9: Typing Indicators

**Goal:** Show when a user is typing in a conversation.

| Item | Details |
|------|---------|
| **Redis** | `typing:{conversation_id}:{user_id}` with 5s TTL |
| **WebSocket Events** | `typing.start`, `typing.stop` |
| **Definition of Done** | Typing indicator appears and disappears correctly |
| **Dependencies** | Phase 5 |
| **Estimated Complexity** | Low (1-2 days) |

---

## Phase 10: Groups

**Goal:** Group creation, membership, roles, group messaging.

| Item | Details |
|------|---------|
| **Database** | Migration: `groups`, `group_members` |
| **APIs** | `POST /groups`, `GET /groups/{id}`, `PATCH /groups/{id}`, `POST/DELETE /groups/{id}/members`, `POST /groups/{id}/leave` |
| **Definition of Done** | Groups with admin/member roles, group messaging, member management |
| **Dependencies** | Phase 6 |
| **Estimated Complexity** | Medium (4-5 days) |

---

## Phase 11: Media

**Goal:** Upload/download images, videos, documents, voice messages.

| Item | Details |
|------|---------|
| **Architecture Changes** | Media module, R2 adapter, thumbnail generation |
| **Database** | Migration: `media_objects`, `message_attachments` |
| **APIs** | `POST /media/upload`, `GET /media/{id}/url` |
| **Definition of Done** | Files upload to R2/MinIO, thumbnails generated for images, signed download URLs work |
| **Dependencies** | Phase 6 |
| **Estimated Complexity** | High (5-7 days) |

---

## Phase 12: Push Notifications

**Goal:** FCM and Web Push notifications.

| Item | Details |
|------|---------|
| **Database** | Migration: `notifications` |
| **Files/Modules** | `src/notifications/`, `src/infrastructure/push.rs` |
| **Definition of Done** | Push notifications arrive for messages and calls when app is backgrounded |
| **Dependencies** | Phase 6 (messages), Phase 3 (devices with push tokens) |
| **Estimated Complexity** | Medium (3-5 days) |

---

## Phase 13: Audio Calls

**Goal:** 1-to-1 audio calling with WebRTC.

| Item | Details |
|------|---------|
| **Architecture Changes** | Call signaling module, TURN credential generation |
| **Database** | Migration: `calls`, `call_participants` |
| **WebSocket Events** | `call.offer`, `call.answer`, `call.ice_candidate`, `call.reject`, `call.end`, `call.ringing`, `call.timeout` |
| **Definition of Done** | Audio calls work P2P and via TURN, call history recorded |
| **Dependencies** | Phase 5 (WebSocket), coturn setup |
| **Estimated Complexity** | High (7-10 days) |

---

## Phase 14: Video Calls

**Goal:** Extend audio calls to include video.

| Item | Details |
|------|---------|
| **Definition of Done** | Video calls work, camera toggle, video quality adequate |
| **Dependencies** | Phase 13 |
| **Estimated Complexity** | Medium (3-4 days) — builds on Phase 13 infrastructure |

---

## Phases 15-20: Production Hardening

| Phase | Goal | Duration Estimate |
|-------|------|-------------------|
| **15: Fault Tolerance** | Supervisor tree, circuit breakers, graceful shutdown | 5-7 days |
| **16: Observability** | Structured logging, metrics, health checks, dashboards | 3-5 days |
| **17: Security Hardening** | Rate limiting tuning, security headers, audit logging, penetration testing | 3-5 days |
| **18: Performance Testing** | Load tests, stress tests, bottleneck identification, optimization | 3-5 days |
| **19: Deployment** | Production deployment pipeline, Cloudflare setup, domain configuration | 3-5 days |
| **20: Production Hardening** | Monitoring alerts, runbooks, backup verification, disaster recovery testing | 3-5 days |

---

## Total Estimated Timeline

| Phases | Description | Estimated Days |
|--------|-------------|---------------|
| 0-1 | Foundation | 3-5 |
| 2-3 | Auth & Users | 8-12 |
| 4-5 | Conversations & WebSocket | 7-10 |
| 6-9 | Messaging & Real-time | 13-19 |
| 10-12 | Groups, Media, Notifications | 12-17 |
| 13-14 | Calls | 10-14 |
| 15-20 | Production Hardening | 18-30 |
| **Total** | | **~71-107 days** |

This assumes a single developer working focused part-time/full-time. Adjust based on actual velocity.

---

*Next: [MVP.md](MVP.md) · [V1.md](V1.md)*
