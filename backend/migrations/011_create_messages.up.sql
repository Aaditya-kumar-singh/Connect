CREATE TABLE messages (
    id UUID PRIMARY KEY,
    client_message_id UUID NOT NULL,
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    sender_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NULL,
    content_type VARCHAR(20) NOT NULL DEFAULT 'text',
    reply_to_message_id UUID NULL REFERENCES messages(id) ON DELETE SET NULL,
    forwarded_from_message_id UUID NULL REFERENCES messages(id) ON DELETE SET NULL,
    edited_at TIMESTAMPTZ NULL,
    deleted_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT messages_content_type_check CHECK (content_type IN ('text', 'image', 'video', 'document', 'voice', 'system')),
    CONSTRAINT messages_client_id_unique UNIQUE (conversation_id, client_message_id)
);

CREATE INDEX idx_messages_conversation_created ON messages(conversation_id, created_at DESC, id DESC);
CREATE INDEX idx_messages_sender ON messages(sender_id);
CREATE INDEX idx_messages_client_msg_id ON messages(conversation_id, client_message_id);

ALTER TABLE conversation_members
    ADD CONSTRAINT fk_conversation_members_last_read_message
    FOREIGN KEY (last_read_message_id) REFERENCES messages(id) ON DELETE SET NULL;
