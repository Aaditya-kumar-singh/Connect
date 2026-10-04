CREATE TABLE message_receipts (
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    delivered_at TIMESTAMPTZ NULL,
    read_at TIMESTAMPTZ NULL,
    PRIMARY KEY (message_id, user_id)
);

CREATE INDEX idx_message_receipts_user_message
    ON message_receipts(user_id, message_id);
