CREATE TABLE schoology_auth (
    id UUID PRIMARY KEY,  -- User's service uuid
    encrypted_email BYTEA NOT NULL,   -- User's schoology google email
    encrypted_password BYTEA NOT NULL,-- User's schoology google password
    session_token BYTEA NULL,              -- User's schoology session token
    FOREIGN KEY (id) REFERENCES service_users(id) ON DELETE CASCADE
);
