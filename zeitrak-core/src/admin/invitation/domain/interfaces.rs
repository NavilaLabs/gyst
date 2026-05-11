use std::fmt::Debug;

use crate::{
    admin::{
        invitation::{self, application::rows::InvitationRow, domain::aggregates::Invitation},
        workspace::WorkspaceId,
    },
    shared::repositories::{ReadRepository, Repository, WriteRepository},
};
use async_trait::async_trait;

use super::aggregates::InvitationId;

#[async_trait]
pub trait InvitationRepository<R>: Repository<Invitation, R> + Send + Sync {
    type Error: Debug
        + Send
        + Sync
        + From<invitation::Error>
        + From<<Self as ReadRepository<Invitation, R>>::Error>
        + From<<Self as WriteRepository<Invitation>>::Error>;

    /// Looks up a pending invitation by its opaque token.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn find_by_token(
        &self,
        token: &str,
    ) -> Result<Option<InvitationRow>, <Self as InvitationRepository<R>>::Error>;

    /// Returns all invitations (any status) for a given workspace.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn find_by_workspace(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<InvitationRow>, <Self as InvitationRepository<R>>::Error>;

    /// Returns all pending invitations for the given email address.
    ///
    /// Used to auto-accept invitations after a new user registers.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn find_pending_for_email(
        &self,
        email: &str,
    ) -> Result<Vec<InvitationId>, <Self as InvitationRepository<R>>::Error>;
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
        admin::invitation::InvitationId,
        shared::{
            AggregateId,
            repositories::{ReadRepository, Repository, RowToRoot, WriteRepository},
        },
    };

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
    impl From<invitation::Error> for StubError {
        fn from(_: invitation::Error) -> Self {
            Self
        }
    }

    impl RowToRoot<(), Invitation> for InMemoryInvitationRepository {
        type Error = StubError;
        fn row_to_root(&self, _row: ()) -> Result<Root<Invitation>, Self::Error> {
            unimplemented!("test stub")
        }
    }

    impl Repository<Invitation, ()> for InMemoryInvitationRepository {}

    #[derive(Debug)]
    pub struct InMemoryInvitationRepository {
        saved: std::sync::Mutex<Vec<Root<Invitation>>>,
    }

    impl InMemoryInvitationRepository {
        pub fn new() -> Self {
            Self {
                saved: std::sync::Mutex::new(vec![]),
            }
        }
    }

    #[async_trait]
    impl Getter<Invitation> for InMemoryInvitationRepository {
        async fn get(&self, id: &InvitationId) -> Result<Root<Invitation>, GetError> {
            self.saved
                .lock()
                .expect("mutex poisoned")
                .iter()
                .find(|r| r.id() == id)
                .cloned()
                .ok_or(GetError::NotFound)
        }
    }

    #[async_trait]
    impl Saver<Invitation> for InMemoryInvitationRepository {
        async fn save(&self, root: &mut Root<Invitation>) -> Result<(), SaveError> {
            {
                let mut store = self.saved.lock().expect("mutex poisoned");
                store.retain(|r| r.id() != root.id());
                store.push(root.clone());
            }
            Ok(())
        }
    }

    #[async_trait]
    impl ReadRepository<Invitation, ()> for InMemoryInvitationRepository {
        type Error = StubError;
        type Filter = ();

        async fn find(&self, _id: AggregateId) -> Result<Option<Root<Invitation>>, StubError> {
            Ok(None)
        }
        async fn find_by(&self, _filter: ()) -> Result<Option<Root<Invitation>>, StubError> {
            Ok(None)
        }
        async fn find_many(
            &self,
            _ids: Vec<AggregateId>,
        ) -> Result<Vec<Root<Invitation>>, StubError> {
            Ok(vec![])
        }
        async fn find_many_by(&self, _filter: ()) -> Result<Vec<Root<Invitation>>, StubError> {
            Ok(vec![])
        }
        async fn all(&self) -> Result<Vec<Root<Invitation>>, StubError> {
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
    impl WriteRepository<Invitation> for InMemoryInvitationRepository {
        type Error = StubError;
    }

    #[async_trait]
    impl InvitationRepository<()> for InMemoryInvitationRepository {
        type Error = StubError;

        async fn find_by_token(&self, _token: &str) -> Result<Option<InvitationRow>, StubError> {
            Ok(None)
        }

        async fn find_by_workspace(
            &self,
            _workspace_id: &WorkspaceId,
        ) -> Result<Vec<InvitationRow>, StubError> {
            Ok(vec![])
        }

        async fn find_pending_for_email(
            &self,
            _email: &str,
        ) -> Result<Vec<InvitationId>, StubError> {
            Ok(vec![])
        }
    }
}
