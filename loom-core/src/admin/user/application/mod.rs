use crate::admin::user::User;

pub mod commands;
pub mod queries;
pub mod rows;

#[derive(Debug)]
#[eventually_macros::aggregate_root(User)]
pub struct UserRoot;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("user not found")]
    UserNotFound,
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("repository error: {0}")]
    RepositoryError(String),
}
