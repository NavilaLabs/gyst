use crate::admin::authenticator::{AuthenticationStrategy, Authenticator, Credentials};
use crate::admin::user::domain::interfaces::UserRepository;

#[derive(Debug, Clone)]
pub struct UserQuery<P> {
    #[allow(dead_code)]
    pool: P,
}

#[derive(Debug, Clone)]
pub struct LoginQuery<P, A>
where
    A: AuthenticationStrategy,
{
    pool: P,
    authenticator: Authenticator<A>,
}

impl<P, A> LoginQuery<P, A>
where
    A: AuthenticationStrategy,
{
    pub const fn new(pool: P, authenticator: Authenticator<A>) -> Self {
        Self {
            pool,
            authenticator,
        }
    }
}

impl<P, A> LoginQuery<P, A>
where
    P: UserRepository,
    A: AuthenticationStrategy,
{
    /// # Errors
    ///
    /// Returns an error if the user is not found, credentials cannot be fetched,
    /// or authentication fails.
    pub async fn login(&self, email: &str, password: &str) -> Result<String, super::Error> {
        let (user_id, stored_email, password_hash) = self
            .pool
            .find_credentials_by_email(email)
            .await
            .map_err(|e| super::Error::RepositoryError(format!("{e:?}")))?
            .ok_or(super::Error::UserNotFound)?;

        self.authenticator
            .authenticate(Credentials {
                user_id: &user_id,
                email: &stored_email,
                password,
                password_hash: &password_hash,
            })
            .map_err(|e| super::Error::AuthenticationFailed(format!("{e:?}")))
    }
}
