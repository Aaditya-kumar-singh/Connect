# Database Migrations — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-DB-003`                               |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-DB-001, DOC-DB-002                    |

---

## 1. Migration File Convention

```
backend/migrations/
├── 001_create_users.up.sql
├── 001_create_users.down.sql
├── 002_create_user_profiles.up.sql
├── 002_create_user_profiles.down.sql
├── 003_create_email_verifications.up.sql
├── 003_create_email_verifications.down.sql
├── 004_create_otp_requests.up.sql
├── 004_create_otp_requests.down.sql
├── 005_create_devices.up.sql
├── 005_create_devices.down.sql
├── 006_create_sessions.up.sql
├── 006_create_sessions.down.sql
├── 007_create_refresh_tokens.up.sql
├── 007_create_refresh_tokens.down.sql
├── 008_create_conversations.up.sql
├── 008_create_conversations.down.sql
├── 009_create_conversation_members.up.sql
├── 009_create_conversation_members.down.sql
├── 010_create_messages.up.sql
├── 010_create_messages.down.sql
├── 011_create_message_receipts.up.sql
├── 011_create_message_receipts.down.sql
├── 012_create_message_reactions.up.sql
├── 012_create_message_reactions.down.sql
├── 013_create_message_edits.up.sql
├── 013_create_message_edits.down.sql
├── 014_create_message_attachments.up.sql
├── 014_create_message_attachments.down.sql
├── 015_create_media_objects.up.sql
├── 015_create_media_objects.down.sql
├── 016_create_groups.up.sql
├── 016_create_groups.down.sql
├── 017_create_group_members.up.sql
├── 017_create_group_members.down.sql
├── 018_create_calls.up.sql
├── 018_create_calls.down.sql
├── 019_create_call_participants.up.sql
├── 019_create_call_participants.down.sql
├── 020_create_notifications.up.sql
├── 020_create_notifications.down.sql
├── 021_create_blocked_users.up.sql
├── 021_create_blocked_users.down.sql
├── 022_create_contacts.up.sql
├── 022_create_contacts.down.sql
└── 023_create_audit_logs.up.sql
└── 023_create_audit_logs.down.sql
```

## 2. Example Migration: Create Users

### `001_create_users.up.sql`

```sql
-- Migration: 001_create_users
-- Creates the users table and related indexes.
-- Dependencies: none

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email           VARCHAR(255) NOT NULL,
    password_hash   VARCHAR(255) NOT NULL,
    display_name    VARCHAR(50)  NOT NULL,
    status          VARCHAR(20)  NOT NULL DEFAULT 'PENDING_VERIFICATION',
    last_seen_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT users_email_unique UNIQUE (email),
    CONSTRAINT users_status_check CHECK (
        status IN ('PENDING_VERIFICATION', 'ACTIVE', 'SUSPENDED', 'DELETED')
    )
);

CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_users_status ON users (status);

-- Trigger: auto-update updated_at on row change
CREATE OR REPLACE FUNCTION trigger_set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();
```

### `001_create_users.down.sql`

```sql
-- Revert: 001_create_users

DROP TRIGGER IF EXISTS set_users_updated_at ON users;
DROP TABLE IF EXISTS users;
-- Note: trigger_set_updated_at function is kept (shared by other tables)
```

## 3. Example Migration: Create Messages

### `010_create_messages.up.sql`

```sql
-- Migration: 010_create_messages
-- Creates the messages table with idempotency constraint.
-- Dependencies: 008_create_conversations, 001_create_users

CREATE TABLE messages (
    id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_message_id         UUID NOT NULL,
    conversation_id           UUID NOT NULL REFERENCES conversations(id),
    sender_id                 UUID NOT NULL REFERENCES users(id),
    content                   TEXT,
    content_type              VARCHAR(20) NOT NULL DEFAULT 'text',
    reply_to_message_id       UUID REFERENCES messages(id) ON DELETE SET NULL,
    forwarded_from_message_id UUID REFERENCES messages(id) ON DELETE SET NULL,
    edited_at                 TIMESTAMPTZ,
    deleted_at                TIMESTAMPTZ,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT messages_idempotency UNIQUE (conversation_id, client_message_id),
    CONSTRAINT messages_content_type_check CHECK (
        content_type IN ('text', 'image', 'video', 'document', 'voice', 'system')
    )
);

-- Primary query: conversation history, paginated by time
CREATE INDEX idx_messages_conversation_created
    ON messages (conversation_id, created_at DESC);

-- Idempotency check (also serves as unique constraint index)
-- Already created by UNIQUE constraint

-- Sender lookup (admin queries, user message history)
CREATE INDEX idx_messages_sender ON messages (sender_id);

-- Trigger: update conversation.updated_at when new message arrives
CREATE OR REPLACE FUNCTION trigger_update_conversation_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE conversations SET updated_at = NOW() WHERE id = NEW.conversation_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_conversation_on_message
    AFTER INSERT ON messages
    FOR EACH ROW
    EXECUTE FUNCTION trigger_update_conversation_timestamp();
```

### `010_create_messages.down.sql`

```sql
DROP TRIGGER IF EXISTS update_conversation_on_message ON messages;
DROP FUNCTION IF EXISTS trigger_update_conversation_timestamp();
DROP TABLE IF EXISTS messages;
```

## 4. Migration Rules

1. **Never modify an existing migration** that has been applied to any environment (including dev databases shared by others).
2. **Always create a new migration** for schema changes. Even to add a single column.
3. **Every UP must have a DOWN.** Test both directions.
4. **Migrations must be idempotent where possible.** Use `IF NOT EXISTS`, `IF EXISTS`.
5. **Data migrations** (transforming existing data) go in separate migration files, not mixed with schema changes.
6. **No destructive changes without explicit approval.** `DROP COLUMN`, `DROP TABLE`, `ALTER TYPE` (shrinking) require backup verification.
7. **Add new columns as nullable** initially. If NOT NULL is required, add with DEFAULT first, then backfill, then add constraint.

## 5. Migration Commands

```bash
# Create a new migration
sqlx migrate add {name} --source backend/migrations

# Apply all pending migrations
sqlx migrate run --source backend/migrations

# Revert the last migration
sqlx migrate revert --source backend/migrations

# Check migration status
sqlx migrate info --source backend/migrations

# Reset database (dev only!)
sqlx database drop -y && sqlx database create && sqlx migrate run --source backend/migrations
```

---

*Next: [schema.md](schema.md) · [backup-recovery.md](backup-recovery.md)*
