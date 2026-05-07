pub mod database;
mod pool;
pub mod read_model;

pub use pool::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SQLx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("SeaORM error: {0}")]
    SeaOrmError(#[from] sea_orm::DbErr),
    #[error("SeaQuery error: {0}")]
    SeaQueryError(#[from] sea_query::error::Error),
    #[error("{0}")]
    MigrateError(#[from] sqlx::migrate::MigrateError),
    #[error("Unsupported database type: {0}")]
    UnsupportedDatabaseType(String),
}

impl From<sqlx::migrate::MigrateError> for crate::Error {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        Error::MigrateError(err).into()
    }
}

impl From<sqlx::Error> for crate::Error {
    fn from(err: sqlx::Error) -> Self {
        Error::SqlxError(err).into()
    }
}

impl From<sea_orm::DbErr> for crate::Error {
    fn from(err: sea_orm::DbErr) -> Self {
        Error::SeaOrmError(err).into()
    }
}

impl From<sea_query::error::Error> for crate::Error {
    fn from(err: sea_query::error::Error) -> Self {
        Error::SeaQueryError(err).into()
    }
}
