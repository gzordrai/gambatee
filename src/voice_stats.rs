use std::str::FromStr;

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

use crate::error::Result;

pub struct VoiceStats {
    pool: SqlitePool,
}

impl VoiceStats {
    pub async fn new(url: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(url)?;
        let pool = SqlitePool::connect_with(options).await?;

        Ok(Self { pool })
    }
}
