pub mod authenticator;
pub mod permission;
pub mod user;
pub mod workspace;
pub mod workspace_role;

use std::fmt::Debug;

use eventually::aggregate::Aggregate;

use crate::shared::repositories::{ReadRepository, WriteRepository};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0:?}")]
    UserError(#[from] user::Error),
    #[error("{0:?}")]
    PermissionError(#[from] permission::Error),
    #[error("{0:?}")]
    WorkspaceError(#[from] workspace::Error),
    #[error("{0:?}")]
    WorkspaceRoleError(#[from] workspace_role::Error),
}

impl<Repo, Row, Agg> From<user::ApplicationError> for crate::Error<Repo, Row, Agg>
where
    Row: Debug,
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Row> + WriteRepository<Agg>,
{
    fn from(value: user::ApplicationError) -> Self {
        Self::AdminDatabaseError(Error::UserError(value.into()))
    }
}

impl<Repo, Row, Agg> From<user::DomainError> for crate::Error<Repo, Row, Agg>
where
    Row: Debug,
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Row> + WriteRepository<Agg>,
{
    fn from(value: user::DomainError) -> Self {
        Self::AdminDatabaseError(Error::UserError(value.into()))
    }
}

impl<Repo, Row, Agg> From<permission::DomainError> for crate::Error<Repo, Row, Agg>
where
    Row: Debug,
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Row> + WriteRepository<Agg>,
{
    fn from(value: permission::DomainError) -> Self {
        Self::AdminDatabaseError(Error::PermissionError(value.into()))
    }
}

impl<Repo, Row, Agg> From<workspace::DomainError> for crate::Error<Repo, Row, Agg>
where
    Row: Debug,
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Row> + WriteRepository<Agg>,
{
    fn from(value: workspace::DomainError) -> Self {
        Self::AdminDatabaseError(Error::WorkspaceError(value.into()))
    }
}

impl<Repo, Row, Agg> From<workspace_role::DomainError> for crate::Error<Repo, Row, Agg>
where
    Row: Debug,
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Row> + WriteRepository<Agg>,
{
    fn from(value: workspace_role::DomainError) -> Self {
        Self::AdminDatabaseError(Error::WorkspaceRoleError(value.into()))
    }
}
