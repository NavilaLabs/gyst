use std::fmt::Debug;

use crate::admin::workspace_role::domain::interfaces::WorkspaceRoleRepository;

pub trait WorkspaceRoleQueryTrait {
    type Error: Debug + Send + Sync;
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

impl<R> WorkspaceRoleQueryTrait for WorkspaceRoleQuery<R>
where
    R: Debug + WorkspaceRoleRepository,
{
    type Error = <R as WorkspaceRoleRepository>::Error;
}
