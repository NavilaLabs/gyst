use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate::Root;

use crate::admin::permission::{
    domain::aggregates::{Permission, PermissionId},
    domain::interfaces::PermissionRepository,
};

#[async_trait]
pub trait PermissionQueryTrait<R> {
    type Error: Debug + Send + Sync;

    async fn find_by_id(&self, id: PermissionId) -> Result<Option<Root<Permission>>, Self::Error>;
    async fn find_all(&self) -> Result<Vec<Root<Permission>>, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct PermissionQuery<Repo> {
    repository: Repo,
}

impl<Repo> PermissionQuery<Repo> {
    pub const fn new(repository: Repo) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<Repo, R> PermissionQueryTrait<R> for PermissionQuery<Repo>
where
    R: Debug + Send + Sync,
    Repo: Debug + Send + Sync + PermissionRepository<R>,
{
    type Error = <Repo as PermissionRepository<R>>::Error;

    async fn find_by_id(&self, id: PermissionId) -> Result<Option<Root<Permission>>, Self::Error> {
        self.repository.find(id).await.map_err(Into::into)
    }

    async fn find_all(&self) -> Result<Vec<Root<Permission>>, Self::Error> {
        self.repository.all().await.map_err(Into::into)
    }
}
