use std::fmt::Debug;

use crate::admin::workspace::domain::interfaces::WorkspaceRepository;

pub trait WorkspaceQueryTrait {
    type Error: Debug + Send + Sync;
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

impl<R> WorkspaceQueryTrait for WorkspaceQuery<R>
where
    R: Debug + WorkspaceRepository,
{
    type Error = <R as WorkspaceRepository>::Error;
}
