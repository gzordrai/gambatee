use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use serenity::all::{User, UserId};
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

        sqlx::migrate!("./migrations").run(&pool).await?;

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

    pub async fn user_left(&self, user: User) -> Result<()> {
        let user_id = user.id;
        let username = user.name;

        if let Some(joined_at) = self.active_sessions.lock().await.remove(&user_id.into()) {
            let duration = Utc::now() - joined_at;
            let user_id = user_id.get() as i64;
            let duration_secs = duration.num_seconds();
            let timestamp = Utc::now();
            let mut tx = self.pool.begin().await?;

            sqlx::query(
                r#"
                INSERT INTO users (user_id, username)
                VALUES ($1, $2)
                ON CONFLICT (user_id) DO UPDATE
                SET username = EXCLUDED.username,
                    updated_at = NOW()
                "#,
            )
            .bind(user_id)
            .bind(username)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                r#"
                INSERT INTO voice_sessions (user_id, duration_seconds, timestamp)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(user_id)
            .bind(duration_secs)
            .bind(timestamp)
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
        }

        Ok(())
    }
}
