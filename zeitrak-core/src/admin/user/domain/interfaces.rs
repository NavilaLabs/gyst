use std::fmt::Debug;

use crate::{
    admin::user::{
        self,
        application::rows::UserRow,
        domain::aggregates::{User, UserId},
    },
    shared::repositories::{ReadRepository, Repository, WriteRepository},
};
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository<R>: Repository<User, R> + Send + Sync {
    type Error: Debug
        + Send
        + Sync
        + From<user::Error>
        + From<<Self as ReadRepository<User, R>>::Error>
        + From<<Self as WriteRepository<User>>::Error>;

    /// Returns `(user_id, email, password)` for the given email — intended
    /// only for authentication flows, not general display.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn find_credentials_by_email(
        &self,
        email: &str,
    ) -> Result<Option<(String, String, String)>, <Self as UserRepository<R>>::Error>;

    /// Returns the view row for the given user ID, or `None` if not found.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn find_view_by_id(
        &self,
        id: &str,
    ) -> Result<Option<UserRow>, <Self as UserRepository<R>>::Error>;

    /// Returns the user ID for the given email address, or `None` if not found.
    ///
    /// Used to detect duplicate email registrations before creating a new user.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn find_id_by_email(
        &self,
        email: &str,
    ) -> Result<Option<UserId>, <Self as UserRepository<R>>::Error>;

    /// Returns the user ID whose `verification_token` matches, or `None`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn find_id_by_verification_token(
        &self,
        token: &str,
    ) -> Result<Option<UserId>, <Self as UserRepository<R>>::Error>;

    /// # Errors
    ///
    /// Returns an error if the database count query fails.
    async fn has_at_least_one_user(&self) -> Result<bool, <Self as UserRepository<R>>::Error> {
        Ok(self.count().await? > 0)
    }
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
        admin::user::UserId,
        shared::{
            AggregateId,
            repositories::{ReadRepository, Repository, RowToRoot, WriteRepository},
        },
    };

    impl RowToRoot<(), User> for InMemoryUserRepository {
        type Error = StubError;
        fn row_to_root(&self, _row: ()) -> Result<Root<User>, Self::Error> {
            unimplemented!("test stub")
        }
    }

    impl Repository<User, ()> for InMemoryUserRepository {}

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

    impl From<user::Error> for StubError {
        fn from(_: user::Error) -> Self {
            Self
        }
    }

    #[derive(Debug)]
    pub struct InMemoryUserRepository;

    impl InMemoryUserRepository {
        pub fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl Getter<User> for InMemoryUserRepository {
        async fn get(&self, _id: &UserId) -> Result<Root<User>, GetError> {
            unimplemented!("test stub")
        }
    }

    #[async_trait]
    impl Saver<User> for InMemoryUserRepository {
        async fn save(&self, _root: &mut Root<User>) -> Result<(), SaveError> {
            Ok(())
        }
    }

    #[async_trait]
    impl ReadRepository<User, ()> for InMemoryUserRepository {
        type Error = StubError;
        type Filter = ();

        async fn find(&self, _id: AggregateId) -> Result<Option<Root<User>>, StubError> {
            Ok(None)
        }
        async fn find_by(&self, _filter: ()) -> Result<Option<Root<User>>, StubError> {
            Ok(None)
        }
        async fn find_many(&self, _ids: Vec<AggregateId>) -> Result<Vec<Root<User>>, StubError> {
            Ok(vec![])
        }
        async fn find_many_by(&self, _filter: ()) -> Result<Vec<Root<User>>, StubError> {
            Ok(vec![])
        }
        async fn all(&self) -> Result<Vec<Root<User>>, StubError> {
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
    impl WriteRepository<User> for InMemoryUserRepository {
        type Error = StubError;
    }

    #[async_trait]
    impl UserRepository<()> for InMemoryUserRepository {
        type Error = StubError;

        async fn find_credentials_by_email(
            &self,
            _email: &str,
        ) -> Result<Option<(String, String, String)>, <Self as UserRepository<()>>::Error> {
            Ok(None)
        }

        async fn find_view_by_id(
            &self,
            _id: &str,
        ) -> Result<Option<crate::admin::user::application::rows::UserRow>, StubError> {
            Ok(None)
        }

        async fn find_id_by_email(&self, _email: &str) -> Result<Option<UserId>, StubError> {
            Ok(None)
        }

        async fn find_id_by_verification_token(
            &self,
            _token: &str,
        ) -> Result<Option<UserId>, StubError> {
            Ok(None)
        }
    }
}
