pub mod authorization;
pub mod config;
pub mod database;
pub mod email;

pub trait ImplError {
    type Error: From<Error> + Send + Sync;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    DateTimeError(#[from] chrono::ParseError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Url(#[from] url::ParseError),
    #[error("{0}")]
    JsonError(#[from] serde_json::Error),
    #[error("{0}")]
    ConfigError(Box<figment::Error>),
    #[error("{0}")]
    DatabaseError(#[from] database::Error),
}

impl From<figment::Error> for Error {
    fn from(e: figment::Error) -> Self {
        Self::ConfigError(Box::new(e))
    }
}
