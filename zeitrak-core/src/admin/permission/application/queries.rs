use std::fmt::Debug;

use crate::admin::permission::domain::interfaces::PermissionRepository;

pub trait PermissionQueryTrait {
    type Error: Debug + Send + Sync;
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

impl<R> PermissionQueryTrait for PermissionQuery<R>
where
    R: Debug + PermissionRepository,
{
    type Error = <R as PermissionRepository>::Error;
}
