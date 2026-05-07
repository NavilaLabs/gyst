use std::fmt::Debug;

use async_trait::async_trait;

use crate::admin::authenticator::{AuthenticationStrategy, Authenticator, Credentials};
use crate::admin::user::domain::interfaces::UserRepository;
use crate::admin::user::{self, User};

pub trait UserQueryTrait {
    type Error: Debug + Send + Sync;
}

#[async_trait]
pub trait LoginQueryTrait {
    type Error: Debug + Send + Sync;

    async fn login(&self, email: &str, password: &str) -> Result<String, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct UserQuery<R> {
    repository: R,
}

impl<R> UserQuery<R> {
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> UserQueryTrait for UserQuery<R>
where
    R: Debug + UserRepository,
{
    type Error = <R as UserRepository>::Error;
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

#[async_trait]
impl<R, A> LoginQueryTrait for LoginQuery<R, A>
where
    R: Debug + UserRepository,
    A: AuthenticationStrategy,
{
    type Error = <R as UserRepository>::Error;

    async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<String, Self::Error> {
        let (user_id, stored_email, password_hash) = self
            .repository
            .find_credentials_by_email(email)
            .await?
            .ok_or(user::Error::NotFound)?;

        Ok(self
            .authenticator
            .authenticate(Credentials {
                user_id: &user_id,
                email: &stored_email,
                password,
                password_hash: &password_hash,
            })
            .map_err(|e| user::Error::AuthenticationFailed(format!("{e:?}")))?)
    }
}
