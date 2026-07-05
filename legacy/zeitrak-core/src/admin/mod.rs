pub mod authenticator;
pub mod invitation;
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
    #[error("{0:?}")]
    InvitationError(#[from] invitation::Error),
}

impl<Repo, Agg, R> From<permission::Error> for crate::Error<Repo, Agg, R>
where
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Agg, R> + WriteRepository<Agg>,
{
    fn from(value: permission::Error) -> Self {
        Self::AdminError(Error::PermissionError(value))
    }
}

impl<Repo, Agg, R> From<user::Error> for crate::Error<Repo, Agg, R>
where
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Agg, R> + WriteRepository<Agg>,
{
    fn from(value: user::Error) -> Self {
        Self::AdminError(Error::UserError(value))
    }
}

impl<Repo, Agg, R> From<workspace::Error> for crate::Error<Repo, Agg, R>
where
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Agg, R> + WriteRepository<Agg>,
{
    fn from(value: workspace::Error) -> Self {
        Self::AdminError(Error::WorkspaceError(value))
    }
}

impl<Repo, Agg, R> From<workspace_role::Error> for crate::Error<Repo, Agg, R>
where
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Agg, R> + WriteRepository<Agg>,
{
    fn from(value: workspace_role::Error) -> Self {
        Self::AdminError(Error::WorkspaceRoleError(value))
    }
}

impl<Repo, Agg, R> From<invitation::Error> for crate::Error<Repo, Agg, R>
where
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Agg, R> + WriteRepository<Agg>,
{
    fn from(value: invitation::Error) -> Self {
        Self::AdminError(Error::InvitationError(value))
    }
}
