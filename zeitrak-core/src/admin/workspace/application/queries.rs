use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate::Root;

use crate::admin::workspace::{
    application::rows::WorkspaceRow,
    domain::aggregates::{Workspace, WorkspaceId},
    domain::interfaces::WorkspaceRepository,
};

#[async_trait]
pub trait WorkspaceQueryTrait<R> {
    type Error: Debug + Send + Sync;

    async fn find_by_id(&self, id: WorkspaceId) -> Result<Option<Root<Workspace>>, Self::Error>;
    async fn find_all(&self) -> Result<Vec<Root<Workspace>>, Self::Error>;
    async fn find_workspaces_for_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<(String, Option<String>)>, Self::Error>;
    async fn find_workspace_for_user(
        &self,
        user_id: &str,
    ) -> Result<Option<String>, Self::Error>;
    async fn find_view_by_id(&self, id: &str) -> Result<Option<WorkspaceRow>, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct WorkspaceQuery<Repo> {
    repository: Repo,
}

impl<Repo> WorkspaceQuery<Repo> {
    pub const fn new(repository: Repo) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<Repo, R> WorkspaceQueryTrait<R> for WorkspaceQuery<Repo>
where
    Repo: Debug + Send + Sync + WorkspaceRepository<R>,
{
    type Error = <Repo as WorkspaceRepository<R>>::Error;

    async fn find_by_id(&self, id: WorkspaceId) -> Result<Option<Root<Workspace>>, Self::Error> {
        self.repository.find(id).await.map_err(Into::into)
    }

    async fn find_all(&self) -> Result<Vec<Root<Workspace>>, Self::Error> {
        self.repository.all().await.map_err(Into::into)
    }

    async fn find_workspaces_for_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<(String, Option<String>)>, Self::Error> {
        self.repository.find_workspaces_for_user(user_id).await
    }

    async fn find_workspace_for_user(
        &self,
        user_id: &str,
    ) -> Result<Option<String>, Self::Error> {
        self.repository.find_workspace_for_user(user_id).await
    }

    async fn find_view_by_id(&self, id: &str) -> Result<Option<WorkspaceRow>, Self::Error> {
        self.repository.find_view_by_id(id).await
    }
}
