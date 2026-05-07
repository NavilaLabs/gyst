use std::fmt::Debug;

pub struct Credentials<'a> {
    pub user_id: &'a str,
    pub email: &'a str,
    /// Plaintext password supplied by the caller.
    pub password: &'a str,
    /// Bcrypt hash stored in the database; the strategy verifies against this.
    pub password_hash: &'a str,
}

pub trait AuthenticationStrategy: Send + Sync {
    type Error: Debug + Send + Sync;

    /// # Errors
    ///
    /// Returns an error if authentication fails.
    fn authenticate(&self, credentials: Credentials<'_>) -> Result<String, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct Authenticator<T: AuthenticationStrategy> {
    strategy: T,
}

impl<T: AuthenticationStrategy> Authenticator<T> {
    pub const fn new(strategy: T) -> Self {
        Self { strategy }
    }

    /// # Errors
    ///
    /// Returns an error if the underlying strategy fails to authenticate.
    pub fn authenticate(&self, credentials: Credentials<'_>) -> Result<String, T::Error> {
        self.strategy.authenticate(credentials)
    }
}
