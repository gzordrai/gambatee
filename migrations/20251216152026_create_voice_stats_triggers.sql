CREATE OR REPLACE FUNCTION update_all_stats()
RETURNS TRIGGER AS $$
DECLARE
    current_year INTEGER := EXTRACT(YEAR FROM NEW.timestamp);
    current_week INTEGER := EXTRACT(WEEK FROM NEW.timestamp);
    current_month INTEGER := EXTRACT(MONTH FROM NEW.timestamp);
    week_start TIMESTAMPTZ := DATE_TRUNC('week', NEW.timestamp);
    week_end TIMESTAMPTZ := week_start + INTERVAL '1 week';
    month_start TIMESTAMPTZ := DATE_TRUNC('month', NEW.timestamp);
    month_end TIMESTAMPTZ := month_start + INTERVAL '1 month';
BEGIN
    INSERT INTO user_voice_stats (user_id, total_seconds, total_sessions, last_session, updated_at)
    VALUES (NEW.user_id, NEW.duration_seconds, 1, NEW.timestamp, NOW())
    ON CONFLICT (user_id) DO UPDATE SET
        total_seconds = user_voice_stats.total_seconds + NEW.duration_seconds,
        total_sessions = user_voice_stats.total_sessions + 1,
        last_session = NEW.timestamp,
        updated_at = NOW();

    INSERT INTO user_weekly_stats (
        user_id, year, week, total_seconds, total_sessions,
        week_start, week_end, updated_at
    )
    VALUES (
        NEW.user_id, current_year, current_week, NEW.duration_seconds, 1,
        week_start, week_end, NOW()
    )
    ON CONFLICT (user_id, year, week) DO UPDATE SET
        total_seconds = user_weekly_stats.total_seconds + NEW.duration_seconds,
        total_sessions = user_weekly_stats.total_sessions + 1,
        updated_at = NOW();

    INSERT INTO user_monthly_stats (
        user_id, year, month, total_seconds, total_sessions,
        month_start, month_end, updated_at
    )
    VALUES (
        NEW.user_id, current_year, current_month, NEW.duration_seconds, 1,
        month_start, month_end, NOW()
    )
    ON CONFLICT (user_id, year, month) DO UPDATE SET
        total_seconds = user_monthly_stats.total_seconds + NEW.duration_seconds,
        total_sessions = user_monthly_stats.total_sessions + 1,
        updated_at = NOW();
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_all_stats
    AFTER INSERT ON voice_sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_all_stats();