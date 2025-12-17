CREATE TABLE IF NOT EXISTS users (
    user_id   BIGINT PRIMARY KEY,
    username  TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_username ON users(username);