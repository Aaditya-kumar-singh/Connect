CREATE TABLE devices (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_name VARCHAR(100) NULL,
    device_type VARCHAR(20) NOT NULL CHECK (device_type IN ('WEB','ANDROID')),
    push_token VARCHAR(500) NULL,
    push_provider VARCHAR(20) NULL CHECK (push_provider IS NULL OR push_provider IN ('FCM','WEB_PUSH')),
    last_active_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_devices_user_id ON devices(user_id);
