use std::fmt::Debug;

use crate::{
    admin::workspace::{
        self,
        application::rows::{MemberRow, WorkspaceRow},
        domain::aggregates::{Workspace, WorkspaceId},
    },
    shared::repositories::{ReadRepository, Repository, WriteRepository},
};
use async_trait::async_trait;

#[async_trait]
pub trait WorkspaceRepository<R>: Repository<Workspace, R> + Send + Sync {
    type Error: Debug
        + Sync
        + Send
        + From<workspace::Error>
        + From<<Self as ReadRepository<Workspace, R>>::Error>
        + From<<Self as WriteRepository<Workspace>>::Error>;

    /// Returns all (`workspace_id`, `workspace_name`) pairs the given user belongs to.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn find_workspaces_for_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<(String, Option<String>)>, <Self as WorkspaceRepository<R>>::Error>;

    /// Returns the first workspace ID the given user belongs to, or `None`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn find_workspace_for_user(
        &self,
        user_id: &str,
    ) -> Result<Option<String>, <Self as WorkspaceRepository<R>>::Error>;

    /// Returns the view row for the given workspace ID, or `None` if not found.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn find_view_by_id(
        &self,
        id: &str,
    ) -> Result<Option<WorkspaceRow>, <Self as WorkspaceRepository<R>>::Error>;

    /// Returns all members of the given workspace with their assigned role and permission IDs.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn find_members(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<MemberRow>, <Self as WorkspaceRepository<R>>::Error>;
}

#[cfg(test)]
#[allow(dead_code)]
pub mod in_memory_repository {
    use async_trait::async_trait;
    use eventually::aggregate::{
        Root,
        repository::{GetError, Getter, SaveError, Saver},
    };

    use super::*;
    use crate::{
        admin::workspace::{WorkspaceId, application::rows::MemberRow},
        shared::{
            AggregateId,
            repositories::{ReadRepository, Repository, RowToRoot, WriteRepository},
        },
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
        fn from(_: GetError) -> Self {
            Self
        }
    }

    impl From<SaveError> for StubError {
        fn from(_: SaveError) -> Self {
            Self
        }
    }

    impl From<workspace::Error> for StubError {
        fn from(_: workspace::Error) -> Self {
            Self
        }
    }

    #[derive(Debug)]
    pub struct InMemoryWorkspaceRepository;

    impl InMemoryWorkspaceRepository {
        pub fn new() -> Self {
            Self
        }
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

        async fn find(&self, _id: AggregateId) -> Result<Option<Root<Workspace>>, StubError> {
            Ok(None)
        }
        async fn find_by(&self, _filter: ()) -> Result<Option<Root<Workspace>>, StubError> {
            Ok(None)
        }
        async fn find_many(
            &self,
            _ids: Vec<AggregateId>,
        ) -> Result<Vec<Root<Workspace>>, StubError> {
            Ok(vec![])
        }
        async fn find_many_by(&self, _filter: ()) -> Result<Vec<Root<Workspace>>, StubError> {
            Ok(vec![])
        }
        async fn all(&self) -> Result<Vec<Root<Workspace>>, StubError> {
            Ok(vec![])
        }
        async fn count_by(&self, _filter: ()) -> Result<u64, StubError> {
            Ok(0)
        }
        async fn count(&self) -> Result<u64, StubError> {
            Ok(0)
        }
    }

    #[async_trait]
    impl WriteRepository<Workspace> for InMemoryWorkspaceRepository {
        type Error = StubError;
    }

    #[async_trait]
    impl WorkspaceRepository<()> for InMemoryWorkspaceRepository {
        type Error = StubError;

        async fn find_workspaces_for_user(
            &self,
            _user_id: &str,
        ) -> Result<Vec<(String, Option<String>)>, StubError> {
            Ok(vec![])
        }

        async fn find_workspace_for_user(
            &self,
            _user_id: &str,
        ) -> Result<Option<String>, StubError> {
            Ok(None)
        }

        async fn find_view_by_id(&self, _id: &str) -> Result<Option<WorkspaceRow>, StubError> {
            Ok(None)
        }

        async fn find_members(
            &self,
            _workspace_id: &WorkspaceId,
        ) -> Result<Vec<MemberRow>, StubError> {
            Ok(vec![])
        }
    }
}
