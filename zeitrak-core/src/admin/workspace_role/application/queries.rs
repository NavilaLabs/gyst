use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate::Root;

use crate::admin::workspace_role::{
    domain::aggregates::{WorkspaceRole, WorkspaceRoleId},
    domain::interfaces::WorkspaceRoleRepository,
};

#[async_trait]
pub trait WorkspaceRoleQueryTrait<R> {
    type Error: Debug + Send + Sync;

    async fn find_by_id(
        &self,
        id: WorkspaceRoleId,
    ) -> Result<Option<Root<WorkspaceRole>>, Self::Error>;
    async fn find_all(&self) -> Result<Vec<Root<WorkspaceRole>>, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct WorkspaceRoleQuery<Repo> {
    repository: Repo,
}

impl<Repo> WorkspaceRoleQuery<Repo> {
    pub const fn new(repository: Repo) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<Repo, R> WorkspaceRoleQueryTrait<R> for WorkspaceRoleQuery<Repo>
where
    R: Debug + Send + Sync,
    Repo: Debug + Send + Sync + WorkspaceRoleRepository<R>,
{
    type Error = <Repo as WorkspaceRoleRepository<R>>::Error;

    async fn find_by_id(
        &self,
        id: WorkspaceRoleId,
    ) -> Result<Option<Root<WorkspaceRole>>, Self::Error> {
        self.repository.find(id).await.map_err(Into::into)
    }

    async fn find_all(&self) -> Result<Vec<Root<WorkspaceRole>>, Self::Error> {
        self.repository.all().await.map_err(Into::into)
    }
}
