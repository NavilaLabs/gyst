use std::fmt::Debug;

use crate::{admin::user::{UserRow, domain::aggregates::User}, shared::repositories::{ReadRepository, WriteRepository}};
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: ReadRepository<UserRow> + WriteRepository<User> + Send + Sync {
    type Error: Debug + From<<Self as ReadRepository<UserRow>>::Error> + From<<Self as WriteRepository<User>>::Error>;

    async fn find_credentials_by_email(
        &self,
        email: &str,
    ) -> Result<Option<(String, String, String)>, <Self as ReadRepository<UserRow>>::Error>;
}
