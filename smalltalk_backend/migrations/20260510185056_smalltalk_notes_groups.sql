CREATE TABLE smalltalk_notes_groups (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    enc_group_name BYTEA NOT NULL,
    enc_group_metadata BYTEA NOT NULL, -- contains colors, and posibly other stuff idk? icon?

    rank INT DEFAULT 0,
    is_deleted BOOLEAN DEFAULT FALSE,
    updated_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,

    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES service_users(id) ON DELETE CASCADE
);

CREATE INDEX idx_groups_user_sync ON smalltalk_notes_groups (user_id, updated_at DESC);
