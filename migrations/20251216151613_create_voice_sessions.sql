CREATE TABLE IF NOT EXISTS voice_sessions (
    id SERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    duration_seconds BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_timestamp ON voice_sessions(timestamp);
CREATE INDEX IF NOT EXISTS idx_user_timestamp ON voice_sessions(user_id, timestamp);
