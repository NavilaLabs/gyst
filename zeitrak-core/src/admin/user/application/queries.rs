use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate::Root;

use crate::admin::authenticator::{AuthenticationStrategy, Authenticator, Credentials};
use crate::admin::user::{
    self,
    application::rows::UserRow,
    domain::aggregates::{User, UserId},
    domain::interfaces::UserRepository,
};

#[async_trait]
pub trait UserQueryTrait<R> {
    type Error: Debug + Send + Sync;

    async fn find_by_id(&self, id: UserId) -> Result<Option<Root<User>>, Self::Error>;
    async fn find_all(&self) -> Result<Vec<Root<User>>, Self::Error>;
    async fn find_view_by_id(&self, id: &str) -> Result<Option<UserRow>, Self::Error>;
    async fn has_at_least_one_user(&self) -> Result<bool, Self::Error>;
    async fn find_id_by_email(&self, email: &str) -> Result<Option<UserId>, Self::Error>;
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

#[async_trait]
impl<Repo, R> UserQueryTrait<R> for UserQuery<Repo>
where
    Repo: Debug + Send + Sync + UserRepository<R>,
{
    type Error = <Repo as UserRepository<R>>::Error;

    async fn find_by_id(&self, id: UserId) -> Result<Option<Root<User>>, Self::Error> {
        self.repository.find(id).await.map_err(Into::into)
    }

    async fn find_all(&self) -> Result<Vec<Root<User>>, Self::Error> {
        self.repository.all().await.map_err(Into::into)
    }

    async fn find_view_by_id(&self, id: &str) -> Result<Option<UserRow>, Self::Error> {
        self.repository.find_view_by_id(id).await
    }

    async fn has_at_least_one_user(&self) -> Result<bool, Self::Error> {
        self.repository.has_at_least_one_user().await
    }

    async fn find_id_by_email(&self, email: &str) -> Result<Option<UserId>, Self::Error> {
        self.repository.find_id_by_email(email).await
    }
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
    Repo: Debug + Send + Sync + UserRepository<R>,
    A: AuthenticationStrategy,
{
    type Error = <Repo as UserRepository<R>>::Error;

    async fn login(&self, email: &str, password: &str) -> Result<String, Self::Error> {
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
