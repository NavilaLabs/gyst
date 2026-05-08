use std::fmt::Debug;

use async_trait::async_trait;

use crate::admin::authenticator::{AuthenticationStrategy, Authenticator, Credentials};
use crate::admin::user::domain::interfaces::UserRepository;
use crate::admin::user::{self};

pub trait UserQueryTrait<R> {
    type Error: Debug + Send + Sync;
}

#[async_trait]
pub trait LoginQueryTrait<R> {
    type Error: Debug + Send + Sync;

    async fn login(&self, email: &str, password: &str) -> Result<String, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct UserQuery<Repo> {
    repository: Repo,
}

impl<Repo> UserQuery<Repo> {
    pub const fn new(repository: Repo) -> Self {
        Self { repository }
    }
}

impl<Repo, R> UserQueryTrait<R> for UserQuery<Repo>
where
    Repo: Debug + UserRepository<R>,
{
    type Error = <Repo as UserRepository<R>>::Error;
}

#[derive(Debug, Clone)]
pub struct LoginQuery<Repo, A>
where
    A: AuthenticationStrategy,
{
    repository: Repo,
    authenticator: Authenticator<A>,
}

impl<Repo, A> LoginQuery<Repo, A>
where
    A: AuthenticationStrategy,
{
    pub const fn new(repository: Repo, authenticator: Authenticator<A>) -> Self {
        Self {
            repository,
            authenticator,
        }
    }
}

#[async_trait]
impl<Repo, R, A> LoginQueryTrait<R> for LoginQuery<Repo, A>
where
    Repo: Debug + UserRepository<R>,
    A: AuthenticationStrategy,
{
    type Error = <Repo as UserRepository<R>>::Error;

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
