CREATE TABLE smalltalk_notes (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    enc_name BYTEA NOT NULL,
    enc_content BYTEA,                    -- have it nullable for when a soft delete happens
    group_id UUID NULL,

    -- password protected note
    is_protected BOOLEAN DEFAULT FALSE,
    password_hash TEXT,                    -- argon2 hash of the note specific password
    salt BYTEA,

    rank INT DEFAULT 0,
    is_deleted BOOLEAN DEFAULT FALSE,
    updated_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,

    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES service_users(id) ON DELETE CASCADE,
    CONSTRAINT fk_group FOREIGN KEY (group_id) REFERENCES smalltalk_notes_groups(id) ON DELETE SET NULL
);

CREATE INDEX idx_notes_user_sync ON smalltalk_notes (user_id, updated_at DESC);
