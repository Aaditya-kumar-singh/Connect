# YBM Connect

> Production-grade, real-time cross-platform communication platform supporting text messaging, media sharing, presence, voice/video calls, and push notifications across Web and Android clients.

---

## 1. Overview

YBM Connect is an independently architected real-time communication platform built on distributed systems principles.

- **Backend:** Modular Monolith in **Rust** (Axum + Tokio)
- **Primary Database:** **PostgreSQL 16** (ACID persistence, 22-table schema)
- **Cache & Pub/Sub:** **Redis 7** (Presence, typing indicators, rate limiting, cross-instance routing)
- **Object Storage:** **Cloudflare R2** / MinIO (S3-compatible media storage)
- **Audio/Video Calls:** **WebRTC** P2P with coturn STUN/TURN fallback
- **Clients:**
  - **Web:** Next.js 14 + TypeScript + Zustand + TanStack Query
  - **Mobile:** Expo / React Native + TypeScript

---

## 2. Documentation

Comprehensive documentation is organized under the [`docs/`](docs/) directory:

- [**Master Engineering Guide**](docs/MASTER-ENGINEERING-GUIDE.md) — Single-page technical reference.
- [**Project Overview**](docs/00-overview/project-overview.md) — Vision, scope, and non-goals.
- [**Architecture Overview**](docs/02-architecture/architecture-overview.md) — System context, data flow, and failure domains.
- [**Database Schema**](docs/10-database/schema.md) — Complete 22-table PostgreSQL schema definition.
- [**Messaging Architecture**](docs/06-messaging/messaging-architecture.md) — Lifecycle, state machines, and idempotency guarantees.
- [**WebRTC Architecture**](docs/09-calls/webrtc.md) — Signaling, STUN/TURN traversal, and call states.
- [**REST API Overview**](docs/12-api/api-overview.md) & [**WebSocket Protocol**](docs/12-api/websocket-protocol.md) — Full API contracts.
- [**Security Architecture**](docs/13-security/security-architecture.md) — Threat model, Argon2id, tokens, and rate limits.
- [**Implementation Roadmap**](docs/22-product/roadmap.md) — 21-phase implementation plan.

---

## 3. Quick Start

### Prerequisites
- [Git](https://git-scm.com/)
- [Node.js](https://nodejs.org/) (LTS $\ge$ 20)
- [Rust](https://rustup.rs/) (Stable $\ge$ 1.75)
- [Docker Desktop](https://www.docker.com/products/docker-desktop/)
- [just](https://github.com/casey/just) command runner (`cargo install just`)

### Local Setup
```bash
# 1. Clone repository
git clone https://github.com/your-org/ybm-connect.git
cd ybm-connect

# 2. Copy environment configuration
cp .env.example .env

# 3. Start infrastructure (PostgreSQL, Redis, MinIO)
docker compose up -d

# 4. Run development workflow
just dev
```

---

## 4. Key Development Commands

| Command | Action |
|---------|--------|
| `just dev` | Start infrastructure, apply migrations, and launch dev servers |
| `just dev-backend` | Run Rust backend with auto-reload (`cargo watch`) |
| `just dev-web` | Run Next.js web frontend |
| `just dev-mobile` | Run Expo mobile app |
| `just db-migrate` | Apply pending database migrations |
| `just test` | Run unit and integration tests |
| `just lint` | Run Clippy and ESLint |
| `just fmt` | Format Rust and frontend source code |

---

## 5. License

Proprietary / All Rights Reserved.
