use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use serenity::all::{User, UserId};
use sqlx::{FromRow, PgPool, postgres::PgConnectOptions};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::error::Result;

#[derive(Debug)]
pub struct VoiceStats {
    pool: PgPool,
    active_sessions: Mutex<HashMap<u64, DateTime<Utc>>>,
}

#[derive(Debug, FromRow)]
pub struct UserStats {
    pub username: String,
    pub total_hours: f64,
    pub total_sessions: i32,
    pub avg_hours_per_session: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum StatsPeriod {
    Weekly,
    Monthly,
}

impl VoiceStats {
    pub async fn new(url: &str) -> Result<Self> {
        info!("Initializing VoiceStats with database");

        let options = PgConnectOptions::from_str(url)?;
        let pool = PgPool::connect_with(options).await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self {
            pool,
            active_sessions: Mutex::new(HashMap::default()),
        })
    }

    pub async fn user_joined(&self, user_id: UserId) {
        let now = Utc::now();

        self.active_sessions.lock().await.insert(user_id.get(), now);
    }

    pub async fn user_left(&self, user: &User) -> Result<()> {
        let user_id = user.id;
        let username = user.name.clone();
        let joined_at = self.active_sessions.lock().await.remove(&user_id.get());

        if let Some(joined_at) = joined_at {
            let now = Utc::now();
            let duration = now - joined_at;
            let duration_secs = duration.num_seconds();

            info!(
                "Recording voice session for user {} - Duration: {}s",
                username, duration_secs
            );

            let user_id_i64 = user_id.get() as i64;
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
            .bind(user_id_i64)
            .bind(&username)
            .execute(&mut *tx)
            .await?;

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

            tx.commit().await?;

            info!("Successfully saved voice session for user {}", username);
        } else {
            warn!(
                "User {} ({}) left but no active session found - possible bot restart or missed join event",
                username, user_id
            );
        }

        Ok(())
    }

    pub async fn get_stats(&self, period: StatsPeriod, limit: i64) -> Result<Vec<UserStats>> {
        let (table, time_field, time_extract) = match period {
            StatsPeriod::Weekly => ("user_weekly_stats", "week", "WEEK"),
            StatsPeriod::Monthly => ("user_monthly_stats", "month", "MONTH"),
        };

        let query = format!(
            r#"
            SELECT
                COALESCE(u.username, s.user_id::text) AS username,
                s.total_seconds / 3600.0 AS total_hours,
                s.total_sessions,
                CASE 
                    WHEN s.total_sessions > 0 
                    THEN (s.total_seconds::float / s.total_sessions) / 3600.0
                    ELSE 0
                END AS avg_hours_per_session
            FROM {} s
            LEFT JOIN users u ON u.user_id = s.user_id
            WHERE s.year = EXTRACT(YEAR FROM NOW())
            AND s.{} = EXTRACT({} FROM NOW())
            ORDER BY s.total_seconds DESC
            LIMIT $1
            "#,
            table, time_field, time_extract
        );

        let stats = sqlx::query_as::<_, UserStats>(&query)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        Ok(stats)
    }
}
