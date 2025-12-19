use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use serenity::all::{User, UserId};
use sqlx::{PgPool, postgres::PgConnectOptions};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::error::Result;

#[derive(Debug)]
pub struct VoiceStats {
    pool: PgPool,
    active_sessions: Mutex<HashMap<u64, DateTime<Utc>>>,
}

impl VoiceStats {
    pub async fn new(url: &str) -> Result<Self> {
        info!("Initializing VoiceStats with database");

        let options = PgConnectOptions::from_str(url)?;
        debug!("Parsed database connection options");

        info!("Connecting to PostgreSQL database...");
        let pool = PgPool::connect_with(options).await?;
        info!("Successfully connected to database");

        info!("Running database migrations...");
        sqlx::migrate!("./migrations").run(&pool).await?;
        info!("Database migrations completed successfully");

        Ok(Self {
            pool,
            active_sessions: Mutex::new(HashMap::default()),
        })
    }

    pub async fn user_joined(&self, user_id: UserId) {
        let now = Utc::now();
        info!(
            "User {} joined voice - recording session start at {}",
            user_id, now
        );

        self.active_sessions.lock().await.insert(user_id.get(), now);

        debug!(
            "Active sessions count: {}",
            self.active_sessions.lock().await.len()
        );
    }

    pub async fn user_left(&self, user: &User) -> Result<()> {
        let user_id = user.id;
        let username = user.name.clone();

        info!(
            "User {} ({}) left voice - processing session",
            user_id, username
        );

        if let Some(joined_at) = self.active_sessions.lock().await.remove(&user_id.into()) {
            let now = Utc::now();
            let duration = now - joined_at;
            let duration_secs = duration.num_seconds();

            info!(
                "Recording voice session for user {} - Duration: {}s",
                username, duration_secs
            );

            let user_id_i64 = user_id.get() as i64;

            debug!("Starting database transaction");
            let mut tx = self.pool.begin().await?;

            debug!("Upserting user {} into database", username);
            sqlx::query(
                r#"
                INSERT INTO users (user_id, username)
                VALUES ($1, $2)
                ON CONFLICT (user_id) DO UPDATE
                SET username = EXCLUDED.username,
                    updated_at = NOW()
                "#,
            )
            .bind(user_id_i64)
            .bind(&username)
            .execute(&mut *tx)
            .await?;

            debug!("Inserting voice session record for user {}", username);
            sqlx::query(
                r#"
                INSERT INTO voice_sessions (user_id, duration_seconds, timestamp)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(user_id_i64)
            .bind(duration_secs)
            .bind(now)
            .execute(&mut *tx)
            .await?;

            debug!("Committing transaction");
            tx.commit().await?;

            info!("Successfully saved voice session for user {}", username);
            debug!(
                "Active sessions count: {}",
                self.active_sessions.lock().await.len()
            );
        } else {
            warn!(
                "User {} ({}) left but no active session found - possible bot restart or missed join event",
                username, user_id
            );
        }

        Ok(())
    }
}
