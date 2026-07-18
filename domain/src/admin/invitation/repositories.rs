use std::fmt::Debug;

use async_trait::async_trait;

use crate::{
    admin::{invitation, workspace},
    repositories::{self, ReadRepository, WriteRepository},
};

#[async_trait]
pub trait Repository<R>: repositories::Repository<invitation::Aggregate, R> + Send + Sync {
    type Error: Debug
        + Send
        + Sync
        + From<invitation::Error>
        + From<<Self as ReadRepository<invitation::Aggregate, R>>::Error>
        + From<<Self as WriteRepository<invitation::Aggregate>>::Error>;

    /// Looks up a pending invitation by its opaque token.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn find_by_token(
        &self,
        token: &str,
    ) -> Result<Option<invitation::Projection>, <Self as Repository<R>>::Error>;

    /// Returns all invitations (any status) for a given workspace.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn find_by_workspace(
        &self,
        workspace_id: &workspace::Id,
    ) -> Result<Vec<invitation::Projection>, <Self as Repository<R>>::Error>;

    /// Returns all pending invitations for the given email address.
    ///
    /// Used to auto-accept invitations after a new user registers.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn find_pending_for_email(
        &self,
        email: &str,
    ) -> Result<Vec<invitation::Id>, <Self as Repository<R>>::Error>;

    /// Returns full invitation rows for all pending, non-expired invitations
    /// addressed to the given email.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn find_all_pending_for_email(
        &self,
        email: &str,
    ) -> Result<Vec<invitation::Projection>, <Self as Repository<R>>::Error>;
}
