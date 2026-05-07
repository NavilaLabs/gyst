use std::fmt::Debug;

use crate::{
    admin::workspace_role::{self, domain::aggregates::WorkspaceRole},
    shared::repositories::{ReadRepository, WriteRepository},
};

pub trait WorkspaceRoleRepository:
    ReadRepository<WorkspaceRole> + WriteRepository<WorkspaceRole> + Send + Sync
{
    type Error: Debug
        + Send
        + Sync
        + From<workspace_role::Error>
        + From<<Self as ReadRepository<WorkspaceRole>>::Error>
        + From<<Self as WriteRepository<WorkspaceRole>>::Error>;
}

#[cfg(test)]
pub mod in_memory_repository {
    use async_trait::async_trait;
    use eventually::aggregate::{
        Root,
        repository::{GetError, Getter, SaveError, Saver},
    };

    use super::*;
    use crate::{
        admin::workspace_role::WorkspaceRoleId,
        shared::{AggregateId, repositories::{ReadRepository, WriteRepository}},
    };

    #[derive(Debug, thiserror::Error)]
    #[error("stub")]
    pub struct StubError;

    impl From<GetError> for StubError {
        fn from(_: GetError) -> Self { Self }
    }

    impl From<SaveError> for StubError {
        fn from(_: SaveError) -> Self { Self }
    }

    impl From<workspace_role::Error> for StubError {
        fn from(_: workspace_role::Error) -> Self { Self }
    }

    #[derive(Debug)]
    pub struct InMemoryWorkspaceRoleRepository;

    impl InMemoryWorkspaceRoleRepository {
        pub fn new() -> Self { Self }
    }

    #[async_trait]
    impl Getter<WorkspaceRole> for InMemoryWorkspaceRoleRepository {
        async fn get(&self, _id: &WorkspaceRoleId) -> Result<Root<WorkspaceRole>, GetError> {
            unimplemented!("test stub")
        }
    }

    #[async_trait]
    impl Saver<WorkspaceRole> for InMemoryWorkspaceRoleRepository {
        async fn save(&self, _root: &mut Root<WorkspaceRole>) -> Result<(), SaveError> {
            Ok(())
        }
    }

    #[async_trait]
    impl ReadRepository<WorkspaceRole> for InMemoryWorkspaceRoleRepository {
        type Error = StubError;
        type Filter = ();

        async fn find(&self, _id: AggregateId) -> Result<Option<Root<WorkspaceRole>>, Self::Error> { Ok(None) }
        async fn find_by(&self, _filter: ()) -> Result<Option<Root<WorkspaceRole>>, Self::Error> { Ok(None) }
        async fn find_many(&self, _ids: Vec<AggregateId>) -> Result<Vec<Root<WorkspaceRole>>, Self::Error> { Ok(vec![]) }
        async fn find_many_by(&self, _filter: ()) -> Result<Vec<Root<WorkspaceRole>>, Self::Error> { Ok(vec![]) }
        async fn all(&self) -> Result<Vec<Root<WorkspaceRole>>, Self::Error> { Ok(vec![]) }
        async fn count_by(&self, _filter: ()) -> Result<u64, Self::Error> { Ok(0) }
        async fn count(&self) -> Result<u64, Self::Error> { Ok(0) }
    }

    #[async_trait]
    impl WriteRepository<WorkspaceRole> for InMemoryWorkspaceRoleRepository {
        type Error = StubError;
    }

    impl WorkspaceRoleRepository for InMemoryWorkspaceRoleRepository {
        type Error = StubError;
    }
}
