CREATE TABLE media_objects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    uploader_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    file_name VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    media_type VARCHAR(20) NOT NULL,
    r2_key VARCHAR(500) NOT NULL UNIQUE,
    thumbnail_r2_key VARCHAR(500) NULL,
    width INTEGER NULL,
    height INTEGER NULL,
    duration_seconds REAL NULL,
    checksum_sha256 VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT media_objects_type_check CHECK (media_type IN ('image','video','document','voice'))
);

CREATE INDEX idx_media_objects_uploader ON media_objects(uploader_id);
