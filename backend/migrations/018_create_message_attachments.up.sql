CREATE TABLE message_attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    media_id UUID NOT NULL REFERENCES media_objects(id),
    position SMALLINT NOT NULL DEFAULT 0,
    CONSTRAINT message_attachments_unique UNIQUE (message_id, media_id),
    CONSTRAINT message_attachments_position_check CHECK (position >= 0)
);

CREATE INDEX idx_message_attachments_message ON message_attachments(message_id);
CREATE INDEX idx_message_attachments_media ON message_attachments(media_id);
