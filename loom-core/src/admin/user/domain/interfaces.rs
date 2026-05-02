use std::fmt::Debug;

use crate::{
    admin::user::{self, domain::aggregates::User},
    shared::repositories::{ReadRepository, WriteRepository},
};
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: ReadRepository<User> + WriteRepository<User> + Send + Sync {
    type Error: Debug
        + Send
        + Sync
        + From<user::Error>
        + From<<Self as ReadRepository<User>>::Error>
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
    ) -> Result<Option<(String, String, String)>, <Self as UserRepository>::Error>;

    /// # Errors
    ///
    /// Returns an error if the database count query fails.
    async fn has_at_least_one_user(&self) -> Result<bool, <Self as UserRepository>::Error> {
        Ok(self.count().await? > 0)
    }
}

#[cfg(test)]
pub mod in_memory_repository {
    use super::*;
    use crate::shared::repositories::in_memory::InMemoryRepository;

    pub struct InMemoryUserRepository {
        inner: InMemoryRepository<User>,
    }

    impl InMemoryUserRepository {
        pub fn new() -> Self {
            Self {
                inner: InMemoryRepository::new(),
            }
        }
    }

    #[async_trait]
    impl UserRepository for InMemoryUserRepository {
        type Error = crate::Error<Self, User>;

        async fn find_credentials_by_email(
            &self,
            email: &str,
        ) -> Result<Option<(String, String, String)>, <Self as UserRepository>::Error> {
            self.inner
                .find(|user| user.email == email)
                .map(|user| Some((user.id.clone(), user.name.clone(), user.email.clone())))
                .map_err(|e| e.into())
        }
    }
}
