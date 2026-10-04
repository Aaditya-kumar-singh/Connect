CREATE TABLE otp_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    otp_type VARCHAR(20) NOT NULL,
    otp_hash VARCHAR(255) NOT NULL,
    attempts SMALLINT NOT NULL DEFAULT 0 CHECK (attempts >= 0 AND attempts <= 5),
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT otp_requests_type_check CHECK (otp_type IN ('PASSWORD_RESET','EMAIL_CHANGE'))
);
CREATE INDEX idx_otp_requests_user_type ON otp_requests(user_id, otp_type);
