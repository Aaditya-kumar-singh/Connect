CREATE TABLE calls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    initiated_by UUID NOT NULL REFERENCES users(id),
    call_type VARCHAR(10) NOT NULL CHECK (call_type IN ('AUDIO','VIDEO')),
    status VARCHAR(15) NOT NULL DEFAULT 'CALLING'
        CHECK (status IN ('CALLING','RINGING','CONNECTING','CONNECTED','ENDED','REJECTED','MISSED','FAILED')),
    started_at TIMESTAMPTZ NULL,
    ended_at TIMESTAMPTZ NULL,
    duration_seconds INTEGER NULL,
    end_reason VARCHAR(30) NULL
        CHECK (end_reason IS NULL OR end_reason IN ('NORMAL','REJECTED','TIMEOUT','ICE_FAILED','ERROR')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE call_participants (
    call_id UUID NOT NULL REFERENCES calls(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    joined_at TIMESTAMPTZ NULL,
    left_at TIMESTAMPTZ NULL,
    PRIMARY KEY (call_id, user_id)
);

CREATE INDEX idx_calls_conversation ON calls(conversation_id, created_at DESC);
CREATE INDEX idx_calls_initiated_by ON calls(initiated_by, created_at DESC);
CREATE INDEX idx_call_participants_user ON call_participants(user_id, call_id);

CREATE INDEX idx_calls_active ON calls(status)
WHERE status IN ('CALLING','RINGING','CONNECTING','CONNECTED');
