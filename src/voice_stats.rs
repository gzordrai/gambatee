use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use serenity::all::UserId;
use sqlx::{PgPool, postgres::PgConnectOptions};
use tokio::sync::Mutex;

use crate::error::Result;

#[derive(Debug)]
pub struct VoiceStats {
    pool: PgPool,
    active_sessions: Mutex<HashMap<u64, DateTime<Utc>>>,
}

impl VoiceStats {
    pub async fn new(url: &str) -> Result<Self> {
        let options = PgConnectOptions::from_str(url)?;
        let pool = PgPool::connect_with(options).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS voice_sessions (
                id SERIAL PRIMARY KEY,
                user_id BIGINT NOT NULL,
                duration_seconds BIGINT NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX IF NOT EXISTS idx_timestamp ON voice_sessions(timestamp);
            CREATE INDEX IF NOT EXISTS idx_user_timestamp ON voice_sessions(user_id, timestamp);
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self {
            pool,
            active_sessions: Mutex::new(HashMap::default()),
        })
    }

    pub async fn user_joined(&self, user_id: UserId) {
        self.active_sessions
            .lock()
            .await
            .insert(user_id.get(), Utc::now());
    }

    pub async fn user_left(&self, user_id: UserId) -> Result<()> {
        if let Some(joined_at) = self.active_sessions.lock().await.remove(&user_id.into()) {
            let duration = Utc::now() - joined_at;
            let user_id = user_id.get() as i64;
            let duration_secs = duration.num_seconds();
            let timestamp = Utc::now().to_rfc3339();

            sqlx::query(
                "INSERT INTO voice_sessions (user_id, duration_seconds, timestamp) 
                 VALUES (?, ?, ?)",
            )
            .bind(user_id)
            .bind(duration_secs)
            .bind(timestamp)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }
}
