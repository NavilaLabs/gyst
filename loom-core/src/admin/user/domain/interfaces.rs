use std::fmt::Debug;

use crate::{admin::user::{domain::aggregates::User}, shared::repositories::{ReadRepository, WriteRepository}};
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: ReadRepository<User> + WriteRepository<User> + Send + Sync {
    type Error: Debug + Sync + Send + From<<Self as ReadRepository<User>>::Error> + From<<Self as WriteRepository<User>>::Error>;

    async fn find_credentials_by_email(
        &self,
        email: &str,
    ) -> Result<Option<(String, String, String)>, <Self as UserRepository>::Error>;
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
        type Error = crate::Error;

        async fn find_credentials_by_email(
            &self,
            email: &str,
        ) -> Result<Option<(String, String, String)>, Self::Error> {
            self.inner
                .find(|user| user.email == email)
                .map(|user| Some((user.id.clone(), user.name.clone(), user.email.clone())))
                .map_err(|e| e.into())
        }
    }
}
