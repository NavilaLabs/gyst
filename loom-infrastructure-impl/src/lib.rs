#[cfg(feature = "sea-query-sqlx")]
mod sea_query_sqlx;
#[cfg(feature = "sea-query-sqlx")]
pub use sea_query_sqlx::*;
pub mod snapshot;

use sqlx::types::uuid;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    PermissionError(#[from] loom_core::admin::permission::Error),
    #[error("{0}")]
    UserError(#[from] loom_core::admin::user::Error),
    #[error("{0}")]
    WorkspaceError(#[from] loom_core::admin::workspace::Error),
    #[error("{0}")]
    WorkspaceRoleError(#[from] loom_core::admin::workspace_role::Error),

    #[error("{0}")]
    GetError(#[from] eventually::aggregate::repository::GetError),
    #[error("{0}")]
    SaveError(#[from] eventually::aggregate::repository::SaveError),
    #[error("{0}")]
    DateTimeError(#[from] chrono::ParseError),
    #[error("{0}")]
    JsonError(#[from] serde_json::Error),
    #[error("{0}")]
    InfrastructureError(#[from] loom_infrastructure::Error),
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[cfg(feature = "sea-query-sqlx")]
    #[error("{0}")]
    SeaQuerySqlxError(#[from] sea_query_sqlx::Error),
    #[error("{0}")]
    UuidError(#[from] uuid::Error),
    #[error("bcrypt error: {0}")]
    BcryptError(#[from] bcrypt::BcryptError),
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("invalid credentials")]
    InvalidCredentials,
}
