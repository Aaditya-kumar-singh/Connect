CREATE TABLE contacts (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    contact_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    nickname VARCHAR(50) NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, contact_user_id),
    CONSTRAINT contacts_not_self CHECK (user_id <> contact_user_id)
);
CREATE INDEX idx_contacts_contact_user ON contacts(contact_user_id);

CREATE TABLE blocked_users (
    blocker_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    blocked_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (blocker_id, blocked_id),
    CONSTRAINT blocked_users_not_self CHECK (blocker_id <> blocked_id)
);
CREATE INDEX idx_blocked_users_blocked ON blocked_users(blocked_id);
