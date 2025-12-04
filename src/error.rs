pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to find: {0}")]
    EnvVar(#[from] std::env::VarError),

    #[error("Failed to load the config: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Failed to ")]
    Client(#[from] serenity::Error),

    #[error("Failed to ")]
    Sqlx(#[from] sqlx::Error),
}
