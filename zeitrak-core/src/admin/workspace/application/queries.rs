use std::fmt::Debug;

use async_trait::async_trait;

use crate::admin::workspace::{domain::interfaces::WorkspaceRepository, Workspace};

#[async_trait]
pub trait WorkspaceQueryTrait {
    type Error: Debug + Send + Sync;

    async fn all(&self) -> Result<Vec<Workspace>, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct WorkspaceQuery<R> {
    repository: R,
}

impl<R> WorkspaceQuery<R> {
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> WorkspaceQueryTrait for WorkspaceQuery<R>
where
    R: Debug + WorkspaceRepository,
{
    type Error = <R as WorkspaceRepository>::Error;

    async fn all(&self) -> Result<Vec<Workspace>, Self::Error> {
        todo!()
    }
}
