use std::fmt::Debug;

use async_trait::async_trait;
use crate::{
    admin::workspace::{self, domain::aggregates::Workspace},
    shared::repositories::{ReadRepository, Repository, WriteRepository},
};

#[async_trait]
pub trait WorkspaceRepository<R>: Repository<Workspace, R> + Send + Sync {
    type Error: Debug
        + Sync
        + Send
        + From<workspace::Error>
        + From<<Self as ReadRepository<Workspace, R>>::Error>
        + From<<Self as WriteRepository<Workspace>>::Error>;
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
        admin::workspace::WorkspaceId,
        shared::{AggregateId, repositories::{ReadRepository, Repository, RowToRoot, WriteRepository}},
    };

    impl RowToRoot<(), Workspace> for InMemoryWorkspaceRepository {
        type Error = StubError;
        fn row_to_root(&self, _row: ()) -> Result<Root<Workspace>, Self::Error> {
            unimplemented!("test stub")
        }
    }

    impl Repository<Workspace, ()> for InMemoryWorkspaceRepository {}

    #[derive(Debug, thiserror::Error)]
    #[error("stub")]
    pub struct StubError;

    impl From<GetError> for StubError {
        fn from(_: GetError) -> Self { Self }
    }

    impl From<SaveError> for StubError {
        fn from(_: SaveError) -> Self { Self }
    }

    impl From<workspace::Error> for StubError {
        fn from(_: workspace::Error) -> Self { Self }
    }

    #[derive(Debug)]
    pub struct InMemoryWorkspaceRepository;

    impl InMemoryWorkspaceRepository {
        pub fn new() -> Self { Self }
    }

    #[async_trait]
    impl Getter<Workspace> for InMemoryWorkspaceRepository {
        async fn get(&self, _id: &WorkspaceId) -> Result<Root<Workspace>, GetError> {
            unimplemented!("test stub")
        }
    }

    #[async_trait]
    impl Saver<Workspace> for InMemoryWorkspaceRepository {
        async fn save(&self, _root: &mut Root<Workspace>) -> Result<(), SaveError> {
            Ok(())
        }
    }

    #[async_trait]
    impl ReadRepository<Workspace, ()> for InMemoryWorkspaceRepository {
        type Error = StubError;
        type Filter = ();

        async fn find(&self, _id: AggregateId) -> Result<Option<Root<Workspace>>, StubError> { Ok(None) }
        async fn find_by(&self, _filter: ()) -> Result<Option<Root<Workspace>>, StubError> { Ok(None) }
        async fn find_many(&self, _ids: Vec<AggregateId>) -> Result<Vec<Root<Workspace>>, StubError> { Ok(vec![]) }
        async fn find_many_by(&self, _filter: ()) -> Result<Vec<Root<Workspace>>, StubError> { Ok(vec![]) }
        async fn all(&self) -> Result<Vec<Root<Workspace>>, StubError> { Ok(vec![]) }
        async fn count_by(&self, _filter: ()) -> Result<u64, StubError> { Ok(0) }
        async fn count(&self) -> Result<u64, StubError> { Ok(0) }
    }

    #[async_trait]
    impl WriteRepository<Workspace> for InMemoryWorkspaceRepository {
        type Error = StubError;
    }

    impl WorkspaceRepository<()> for InMemoryWorkspaceRepository {
        type Error = StubError;
    }
}
