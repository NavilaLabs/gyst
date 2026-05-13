use std::fmt::Debug;

use async_trait::async_trait;

use crate::admin::{
    invitation::{
        application::rows::InvitationRow,
        domain::{aggregates::InvitationId, interfaces::InvitationRepository},
    },
    workspace::WorkspaceId,
};

#[async_trait]
pub trait InvitationQueryTrait<R> {
    type Error: Debug + Send + Sync;

    async fn find_by_token(&self, token: &str) -> Result<Option<InvitationRow>, Self::Error>;
    async fn find_by_workspace(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<InvitationRow>, Self::Error>;
    async fn find_pending_for_email(&self, email: &str) -> Result<Vec<InvitationId>, Self::Error>;
    async fn find_all_pending_for_email(
        &self,
        email: &str,
    ) -> Result<Vec<InvitationRow>, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct InvitationQuery<Repo> {
    repository: Repo,
}

impl<Repo> InvitationQuery<Repo> {
    pub const fn new(repository: Repo) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<Repo, R> InvitationQueryTrait<R> for InvitationQuery<Repo>
where
    Repo: Debug + Send + Sync + InvitationRepository<R>,
{
    type Error = <Repo as InvitationRepository<R>>::Error;

    async fn find_by_token(&self, token: &str) -> Result<Option<InvitationRow>, Self::Error> {
        self.repository.find_by_token(token).await
    }

    async fn find_by_workspace(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<InvitationRow>, Self::Error> {
        self.repository.find_by_workspace(workspace_id).await
    }

    async fn find_pending_for_email(&self, email: &str) -> Result<Vec<InvitationId>, Self::Error> {
        self.repository.find_pending_for_email(email).await
    }

    async fn find_all_pending_for_email(
        &self,
        email: &str,
    ) -> Result<Vec<InvitationRow>, Self::Error> {
        self.repository.find_all_pending_for_email(email).await
    }
}
