CREATE TABLE IF NOT EXISTS voice_sessions (
    id SERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    duration_seconds BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_voice_stats (
    user_id BIGINT PRIMARY KEY,
    total_seconds BIGINT NOT NULL DEFAULT 0,
    total_sessions INTEGER NOT NULL DEFAULT 0,
    last_session TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_weekly_stats (
    id SERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    year INTEGER NOT NULL,
    week INTEGER NOT NULL,
    total_seconds BIGINT NOT NULL DEFAULT 0,
    total_sessions INTEGER NOT NULL DEFAULT 0,
    week_start TIMESTAMPTZ NOT NULL,
    week_end TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, year, week)
);

CREATE TABLE IF NOT EXISTS user_monthly_stats (
    id SERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    year INTEGER NOT NULL,
    month INTEGER NOT NULL,
    total_seconds BIGINT NOT NULL DEFAULT 0,
    total_sessions INTEGER NOT NULL DEFAULT 0,
    month_start TIMESTAMPTZ NOT NULL,
    month_end TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, year, month)
);

CREATE INDEX idx_sessions_timestamp ON voice_sessions(timestamp);
CREATE INDEX idx_sessions_user_time ON voice_sessions(user_id, timestamp);
CREATE INDEX idx_stats_total ON user_voice_stats(total_seconds DESC);
CREATE INDEX idx_weekly_user_year_week ON user_weekly_stats(user_id, year, week);
CREATE INDEX idx_weekly_year_week ON user_weekly_stats(year DESC, week DESC);
CREATE INDEX idx_weekly_total ON user_weekly_stats(total_seconds DESC);
CREATE INDEX idx_monthly_user_year_month ON user_monthly_stats(user_id, year, month);
CREATE INDEX idx_monthly_year_month ON user_monthly_stats(year DESC, month DESC);
CREATE INDEX idx_monthly_total ON user_monthly_stats(total_seconds DESC);