pub mod config;
pub mod database;

pub trait ImplError {
    type Error: From<Error> + Send + Sync;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    DateTimeError(#[from] chrono::ParseError),
    #[error("{0}")]
    EnvVarError(#[from] dotenvy::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Url(#[from] url::ParseError),
    #[error("{0}")]
    YamlError(#[from] serde_yaml::Error),
    #[error("{0}")]
    JsonError(#[from] serde_json::Error),
    #[error("{0}")]
    ConfigError(#[from] config::Error),
    #[error("{0}")]
    DatabaseError(#[from] database::Error),
}
