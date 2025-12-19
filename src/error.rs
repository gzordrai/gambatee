pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to find environment variable: {0}")]
    EnvVar(#[from] std::env::VarError),

    #[error("Failed to load the config: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Serenity error: {0}")]
    Serenity(#[from] Box<serenity::Error>),

    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Migration failed: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Failed to set global default tracing subscriber: {0}")]
    SetGlobalDefault(#[from] tracing::subscriber::SetGlobalDefaultError),
}

impl From<serenity::Error> for Error {
    fn from(err: serenity::Error) -> Self {
        Error::Serenity(Box::new(err))
    }
}
