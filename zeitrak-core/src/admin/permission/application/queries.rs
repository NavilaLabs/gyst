use std::fmt::Debug;

use async_trait::async_trait;

use crate::admin::permission::{domain::interfaces::PermissionRepository, Permission};

#[async_trait]
pub trait PermissionQueryTrait {
    type Error: Debug + Send + Sync;

    async fn all(&self) -> Result<Vec<Permission>, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct PermissionQuery<R> {
    repository: R,
}

impl<R> PermissionQuery<R> {
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> PermissionQueryTrait for PermissionQuery<R>
where
    R: Debug + PermissionRepository,
{
    type Error = <R as PermissionRepository>::Error;

    async fn all(&self) -> Result<Vec<Permission>, Self::Error> {
        todo!()
    }
}
