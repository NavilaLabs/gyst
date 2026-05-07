use std::fmt::Debug;

use async_trait::async_trait;

use crate::admin::workspace_role::{domain::interfaces::WorkspaceRoleRepository, WorkspaceRole};

#[async_trait]
pub trait WorkspaceRoleQueryTrait {
    type Error: Debug + Send + Sync;

    async fn all(&self) -> Result<Vec<WorkspaceRole>, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct WorkspaceRoleQuery<R> {
    repository: R,
}

impl<R> WorkspaceRoleQuery<R> {
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> WorkspaceRoleQueryTrait for WorkspaceRoleQuery<R>
where
    R: Debug + WorkspaceRoleRepository,
{
    type Error = <R as WorkspaceRoleRepository>::Error;

    async fn all(&self) -> Result<Vec<WorkspaceRole>, Self::Error> {
        todo!()
    }
}
