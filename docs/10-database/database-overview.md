# Database Overview — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-DB-001`                               |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-DB-002 through 007                    |
| **Related ADRs**  | ADR-004                                    |

---

## 1. Why PostgreSQL

| Requirement | PostgreSQL Capability |
|-------------|----------------------|
| Relational data (users, conversations, membership) | Full SQL with foreign keys, joins, constraints |
| ACID transactions | Full transaction support with isolation levels |
| JSON data (notification payloads, audit details) | Native JSONB type with indexing |
| Full-text search (future: message search) | Built-in `tsvector` and `GIN` indexes |
| UUID support | Native `uuid` type, `gen_random_uuid()` |
| Timestamp with timezone | Native `TIMESTAMPTZ` |
| Connection pooling | Via PgBouncer or sqlx pool |
| Mature ecosystem | Well-documented, battle-tested, extensive tooling |

## 2. Connection Pool Configuration

```rust
let pool = PgPoolOptions::new()
    .max_connections(config.database_max_connections) // Default: 10
    .min_connections(config.database_min_connections) // Default: 2
    .acquire_timeout(Duration::from_secs(5))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .test_before_acquire(true)
    .connect(&config.database_url)
    .await?;
```

## 3. Migration Strategy

Migrations use sqlx-cli with explicit UP and DOWN files:

```
backend/migrations/
├── 001_create_users.up.sql
├── 001_create_users.down.sql
├── 002_create_sessions.up.sql
├── 002_create_sessions.down.sql
├── 003_create_conversations.up.sql
├── 003_create_conversations.down.sql
└── ...
```

### Rules
1. Migrations are numbered sequentially
2. Every UP has a corresponding DOWN
3. DOWN migrations must be tested
4. Never modify an existing migration after it has been applied to any environment
5. Destructive migrations (DROP COLUMN, DROP TABLE) require explicit approval and backup verification

## 4. Table Summary

| Table | Records (Expected) | Key Queries | Primary Index |
|-------|--------------------|----|---|
| `users` | 1K-100K | Login by email, lookup by ID | PK: id, UNIQUE: email |
| `user_profiles` | 1:1 with users | Profile view | PK: user_id (FK) |
| `email_verifications` | Transient | Latest for user | idx: user_id |
| `otp_requests` | Transient | Rate check by user+type | idx: (user_id, otp_type) |
| `sessions` | ~3x users | Active sessions per user | idx: user_id, partial: active only |
| `devices` | ~2x users | Devices per user | idx: user_id |
| `refresh_tokens` | ~1x sessions | Token lookup, family check | UNIQUE: token_hash, idx: family_id |
| `conversations` | ~10x users | By ID | PK: id |
| `conversation_members` | ~20x users | User's conversations, conv members | idx: user_id, idx: conversation_id |
| `messages` | 100K-10M+ | Conversation history (paginated) | idx: (conversation_id, created_at DESC) |
| `message_receipts` | ~1x messages | Receipt status | PK: (message_id, user_id) |
| `message_reactions` | ~0.1x messages | Reactions per message | PK: (message_id, user_id, emoji) |
| `message_edits` | ~0.01x messages | Edit history | idx: message_id |
| `message_attachments` | ~0.1x messages | Attachments per message | idx: message_id |
| `media_objects` | ~0.1x messages | By ID, by uploader | PK: id, UNIQUE: r2_key |
| `groups` | ~0.05x conversations | By conversation | UNIQUE: conversation_id |
| `group_members` | ~10x groups | Members per group | PK: (group_id, user_id) |
| `calls` | ~1K-50K | Call history | idx: conversation_id |
| `call_participants` | ~2x calls | Participants | PK: (call_id, user_id) |
| `notifications` | ~2x messages | Per user, paginated | idx: (user_id, created_at DESC) |
| `blocked_users` | ~0.01x users | Check blocks | PK: (blocker_id, blocked_id) |
| `contacts` | ~10x users | User's contacts | PK: (user_id, contact_user_id) |
| `audit_logs` | Continuous growth | By user, by action, by time | idx: user_id, action, created_at |

Full schema with columns: [schema.md](schema.md)

---

*Next: [schema.md](schema.md) · [migrations.md](migrations.md)*
