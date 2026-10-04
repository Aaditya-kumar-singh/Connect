CREATE TABLE groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description VARCHAR(500) NULL,
    avatar_url VARCHAR(500) NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL,
    CONSTRAINT groups_conversation_unique UNIQUE (conversation_id)
);

CREATE INDEX idx_groups_created_by ON groups(created_by);
CREATE INDEX idx_groups_active ON groups(id) WHERE deleted_at IS NULL;

ALTER TABLE messages ALTER COLUMN sender_id DROP NOT NULL;
