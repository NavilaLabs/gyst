use std::fmt::Debug;

use async_trait::async_trait;
use chrono::Utc;
use eventually::aggregate::Root;
use uuid::Uuid;

use crate::admin::{
    invitation::{
        application::InvitationRoot,
        domain::{
            aggregates::{Invitation, InvitationId},
            events::InvitationEvent,
            interfaces::InvitationRepository,
        },
    },
    user::UserId,
    workspace::WorkspaceId,
    workspace_role::WorkspaceRoleId,
};

#[async_trait]
pub trait InvitationCommandTrait<R> {
    type Error: Debug + Sync + Send;

    async fn create(
        &self,
        id: InvitationId,
        workspace_id: WorkspaceId,
        invited_by: UserId,
        email: String,
        workspace_role_id: WorkspaceRoleId,
        ttl_days: u32,
    ) -> Result<Root<Invitation>, Self::Error>;

    async fn accept(&self, id: InvitationId, accepted_by: UserId) -> Result<(), Self::Error>;

    async fn revoke(&self, id: InvitationId) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub struct InvitationCommand<Repo> {
    repository: Repo,
}

impl<Repo> InvitationCommand<Repo> {
    pub const fn new(repository: Repo) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<Repo, R> InvitationCommandTrait<R> for InvitationCommand<Repo>
where
    Repo: Debug + InvitationRepository<R>,
{
    type Error = crate::Error<Repo, Invitation, R>;

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied or the root cannot be saved.
    async fn create(
        &self,
        id: InvitationId,
        workspace_id: WorkspaceId,
        invited_by: UserId,
        email: String,
        workspace_role_id: WorkspaceRoleId,
        ttl_days: u32,
    ) -> Result<Root<Invitation>, Self::Error> {
        let token = Uuid::now_v7().to_string();
        let expires_at = Utc::now() + chrono::Duration::days(i64::from(ttl_days));

        let mut root = Root::<Invitation>::record_new(
            InvitationEvent::Created {
                id,
                workspace_id,
                invited_by,
                email,
                workspace_role_id,
                token,
                expires_at,
            }
            .into(),
        )?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))?;
        Ok(root)
    }

    /// # Errors
    ///
    /// Returns an error if the invitation is not pending, is expired, or cannot be saved.
    async fn accept(&self, id: InvitationId, accepted_by: UserId) -> Result<(), Self::Error> {
        let mut root: InvitationRoot = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();

        if !root.is_pending() {
            return Err(crate::Error::AdminError(
                crate::admin::Error::InvitationError(crate::admin::invitation::Error::NotPending),
            ));
        }
        if root.is_expired() {
            return Err(crate::Error::AdminError(
                crate::admin::Error::InvitationError(crate::admin::invitation::Error::Expired),
            ));
        }

        root.record_that(InvitationEvent::Accepted { accepted_by }.into())?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
    }

    /// # Errors
    ///
    /// Returns an error if the invitation cannot be saved.
    async fn revoke(&self, id: InvitationId) -> Result<(), Self::Error> {
        let mut root: InvitationRoot = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();

        root.record_that(InvitationEvent::Revoked.into())?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::invitation::domain::interfaces::in_memory_repository::InMemoryInvitationRepository;

    fn workspace_id() -> WorkspaceId {
        "019d0ce8-facb-7c90-b9d7-000000000002"
            .parse()
            .expect("valid UUID")
    }

    fn user_id() -> UserId {
        "019d0ce8-facb-7c90-b9d7-000000000003"
            .parse()
            .expect("valid UUID")
    }

    fn role_id() -> WorkspaceRoleId {
        "019d0ce8-facb-7c90-b9d7-000000000004"
            .parse()
            .expect("valid UUID")
    }

    #[tokio::test]
    async fn create_returns_pending_invitation() {
        let repo = InMemoryInvitationRepository::new();
        let cmd = InvitationCommand::new(repo);
        let id = InvitationId::new();

        let result = cmd
            .create(
                id,
                workspace_id(),
                user_id(),
                "bob@example.com".to_string(),
                role_id(),
                7,
            )
            .await;

        assert!(result.is_ok());
        let root = result.unwrap();
        assert!(root.is_pending());
        assert_eq!(root.email(), "bob@example.com");
    }

    #[tokio::test]
    async fn create_generates_non_empty_token() {
        let repo = InMemoryInvitationRepository::new();
        let cmd = InvitationCommand::new(repo);
        let id = InvitationId::new();

        let root = cmd
            .create(
                id,
                workspace_id(),
                user_id(),
                "bob@example.com".to_string(),
                role_id(),
                7,
            )
            .await
            .unwrap();

        assert!(!root.token().is_empty());
    }
}
