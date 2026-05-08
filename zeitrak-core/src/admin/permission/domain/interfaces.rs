use std::fmt::Debug;

use async_trait::async_trait;
use crate::{
    admin::permission::{self, domain::aggregates::Permission},
    shared::repositories::{ReadRepository, Repository, WriteRepository},
};

#[async_trait]
pub trait PermissionRepository<R>: Repository<Permission, R> + Send + Sync {
    type Error: Debug
        + Send
        + Sync
        + From<permission::Error>
        + From<<Self as ReadRepository<Permission, R>>::Error>
        + From<<Self as WriteRepository<Permission>>::Error>;
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
        admin::permission::{self, domain::aggregates::PermissionId},
        shared::{AggregateId, repositories::{ReadRepository, Repository, RowToRoot, WriteRepository}},
    };

    impl RowToRoot<(), Permission> for InMemoryPermissionRepository {
        type Error = StubError;
        fn row_to_root(&self, _row: ()) -> Result<Root<Permission>, Self::Error> {
            unimplemented!("test stub")
        }
    }

    impl Repository<Permission, ()> for InMemoryPermissionRepository {}

    #[derive(Debug, thiserror::Error)]
    #[error("stub")]
    pub struct StubError;

    impl From<GetError> for StubError {
        fn from(_: GetError) -> Self { Self }
    }

    impl From<SaveError> for StubError {
        fn from(_: SaveError) -> Self { Self }
    }

    impl From<permission::Error> for StubError {
        fn from(_: permission::Error) -> Self { Self }
    }

    #[derive(Debug)]
    pub struct InMemoryPermissionRepository;

    impl InMemoryPermissionRepository {
        pub fn new() -> Self { Self }
    }

    #[async_trait]
    impl Getter<Permission> for InMemoryPermissionRepository {
        async fn get(&self, _id: &PermissionId) -> Result<Root<Permission>, GetError> {
            unimplemented!("test stub")
        }
    }

    #[async_trait]
    impl Saver<Permission> for InMemoryPermissionRepository {
        async fn save(&self, _root: &mut Root<Permission>) -> Result<(), SaveError> {
            Ok(())
        }
    }

    #[async_trait]
    impl ReadRepository<Permission, ()> for InMemoryPermissionRepository {
        type Error = StubError;
        type Filter = ();

        async fn find(&self, _id: AggregateId) -> Result<Option<Root<Permission>>, StubError> { Ok(None) }
        async fn find_by(&self, _filter: ()) -> Result<Option<Root<Permission>>, StubError> { Ok(None) }
        async fn find_many(&self, _ids: Vec<AggregateId>) -> Result<Vec<Root<Permission>>, StubError> { Ok(vec![]) }
        async fn find_many_by(&self, _filter: ()) -> Result<Vec<Root<Permission>>, StubError> { Ok(vec![]) }
        async fn all(&self) -> Result<Vec<Root<Permission>>, StubError> { Ok(vec![]) }
        async fn count_by(&self, _filter: ()) -> Result<u64, StubError> { Ok(0) }
        async fn count(&self) -> Result<u64, StubError> { Ok(0) }
    }

    #[async_trait]
    impl WriteRepository<Permission> for InMemoryPermissionRepository {
        type Error = StubError;
    }

    impl PermissionRepository<()> for InMemoryPermissionRepository {
        type Error = StubError;
    }
}
