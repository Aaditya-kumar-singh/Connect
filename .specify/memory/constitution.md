<!--
Sync Impact Report:
- Version change: [CONSTITUTION_VERSION] -> 1.0.0
- List of modified principles:
  - [PRINCIPLE_1_NAME] -> I. Modular Monolith Architecture & Strict Layering
  - [PRINCIPLE_2_NAME] -> II. Durability, Idempotency & Message State Machine
  - [PRINCIPLE_3_NAME] -> III. Ephemeral Real-Time Coordination via Redis
  - [PRINCIPLE_4_NAME] -> IV. Zero-Trust Security & Defense in Depth
  - [PRINCIPLE_5_NAME] -> V. Test-First Quality & Contract Verification
- Added sections:
  - Technical Constraints & Stack Requirements
  - Development Workflow & Quality Gates
- Removed sections: None
- Deferred items / follow-up TODOs: None
-->

# YBM Connect Constitution

## Core Principles

### I. Modular Monolith Architecture & Strict Layering
The backend is structured as a single modular Rust binary (Axum + Tokio) with strict internal separation of concerns:
- **Layering Order:** Transport (Axum handlers) → Service (business logic) → Repository (data access) → Infrastructure (PostgreSQL, Redis, R2).
- Business logic MUST only reside in the Service layer. Domain types are shared, but handlers must never access repositories directly.
- **Extraction Rule:** Only split a module into a separate service if there is an independent scaling requirement, separate failure domain, or distinct protocol/runtime boundary (e.g., coturn for UDP STUN/TURN).

### II. Durability, Idempotency & Message State Machine
PostgreSQL 16 is the authoritative source of truth for all persistent state.
- **Idempotency:** Every client message MUST use a client-generated UUID v7 as `client_message_id`. Server operations enforce `INSERT ... ON CONFLICT DO NOTHING` to prevent duplicate processing.
- **Durability Guarantee:** A message acknowledgement (`message.ack`) MUST be sent to the sender ONLY AFTER the PostgreSQL transaction commits successfully.
- **Strict State Machine:** Message lifecycle follows `CREATED → SENT → VALIDATED → PERSISTED → ACKNOWLEDGED → DELIVERED → READ`. Server timestamps govern message ordering.
- Soft-delete semantics are enforced for user-facing deletion to preserve referential integrity and audit trails.

### III. Ephemeral Real-Time Coordination via Redis
Redis 7 is strictly reserved for ephemeral, low-latency coordination:
- Used exclusively for real-time presence, typing indicators, sliding-window rate limiting, and cross-instance WebSocket message routing via Pub/Sub.
- Ephemeral state MUST NEVER be treated as durable. If Redis restarts or drops data, the system must self-heal and sync from PostgreSQL without data loss.

### IV. Zero-Trust Security & Defense in Depth
Security controls must be applied at every layer of the architecture:
- Passwords MUST be hashed using Argon2id (`m=19456, t=2, p=1`).
- Plaintext authentication tokens, session secrets, or API keys MUST NEVER be persisted in the database; only cryptographic hashes (SHA-256) are stored.
- Audit logs are append-only; update and delete privileges are revoked at the database level.
- Multi-tier rate limiting (per-IP, per-user, per-endpoint) is mandatory for all public and authenticated endpoints.

### V. Test-First Quality & Contract Verification
Quality is verified continuously through automated, deterministic tests:
- Core domain logic, state machines, and cryptographic flows require comprehensive unit test coverage.
- Repositories and external integrations require contract integration tests verified against real PostgreSQL and Redis containers.
- Backward compatibility of REST and WebSocket schemas is non-negotiable; all schema changes must be accompanied by reversible migrations.

## Technical Constraints & Stack Requirements

- **Backend:** Rust (Stable $\ge$ 1.75), Tokio async runtime, Axum web framework, SQLx, Redis-rs. Rust is the primary application backend language for all business logic, API endpoints, data access, and client-facing protocols.
- **Reliability Runtime:** Erlang/OTP 27 (rebar3 build tool). Used exclusively for dedicated reliability/fault-tolerance responsibilities: OTP supervision trees, dependency health monitoring, circuit breakers, recovery coordination. Erlang/OTP must not be used for application business logic. See ADR-018 for rationale.
- **Database:** PostgreSQL 16 (22-table normalized schema with strict foreign keys and indexes).
- **Cache & Pub/Sub:** Redis 7 (cluster-ready, standalone in development). Also used for Rust↔Erlang reliability event exchange.
- **Object Storage:** Cloudflare R2 / MinIO (S3-compatible API for encrypted media storage).
- **Audio/Video Calls:** WebRTC P2P mesh for 1:1 calls with coturn STUN/TURN traversal.
- **Frontend Clients:**
  - Web: Next.js 14, TypeScript, Zustand, TanStack Query.
  - Mobile: Expo / React Native, TypeScript.
- **Tooling:** `just` command runner, Docker Compose for local service orchestration.

## Development Workflow & Quality Gates

- **Specification Before Code:** All new features or architectural modifications must start with a Spec Kit specification (`.specify/specs/`) and approved implementation plan before writing code.
- **Automated Verification:**
  - Code formatting: `just fmt` (`cargo fmt` and `prettier`).
  - Static analysis: `just lint` (`cargo clippy -- -D warnings` and `eslint`).
  - Automated test suite: `just test` (`cargo test`).
  - Erlang tests: `rebar3 eunit` and `rebar3 ct` (for reliability service).
  - Migration validation: `just db-migrate`.
- **Git & Branching Hygiene:** Feature branches must follow the Spec Kit feature naming conventions (`###-feature-name`), keeping commits focused, atomic, and traceable to spec tasks.

## Governance

- The Constitution supersedes all ad-hoc conventions and undocumented practices.
- Any amendment to these principles requires an updated revision in `.specify/memory/constitution.md`, a semantic version bump, and team review.
- Compliance is verified during Spec Kit analysis (`speckit-analyze`), pull request reviews, and CI automation.

**Version**: 1.1.0 | **Ratified**: 2026-10-02 | **Last Amended**: 2026-10-04
