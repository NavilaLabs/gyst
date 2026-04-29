use std::fmt::Debug;

use async_trait::async_trait;

use crate::admin::authenticator::{AuthenticationStrategy, Authenticator, Credentials};
use crate::admin::user::{self, User};
use crate::admin::user::domain::interfaces::UserRepository;

#[async_trait]
pub trait UserQueryTrait {
    type Error: Debug;

    async fn login(&self, email: &str, password: &str) -> Result<String, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct UserQuery<R> {
    repository: R,
}

#[derive(Debug, Clone)]
pub struct LoginQuery<R, A>
where
    A: AuthenticationStrategy,
{
    repository: R,
    authenticator: Authenticator<A>,
}

impl<R, A> LoginQuery<R, A>
where
    A: AuthenticationStrategy,
{
    pub const fn new(repository: R, authenticator: Authenticator<A>) -> Self {
        Self {
            repository,
            authenticator,
        }
    }
}

impl<R, A> LoginQuery<R, A>
where
    R: UserRepository<Error = crate::Error<R, User>>,
    A: AuthenticationStrategy,
{
    /// # Errors
    ///
    /// Returns an error if the user is not found, credentials cannot be fetched,
    /// or authentication fails.
    pub async fn login(&self, email: &str, password: &str) -> Result<String, crate::Error<R, User>> {
        let (user_id, stored_email, password_hash) = self
            .repository
            .find_credentials_by_email(email)
            .await?
            .ok_or(user::Error::NotFound)?;

        Ok(self.authenticator
            .authenticate(Credentials {
                user_id: &user_id,
                email: &stored_email,
                password,
                password_hash: &password_hash,
            })
            .map_err(|e| user::Error::AuthenticationFailed(format!("{e:?}")))?)
    }
}
