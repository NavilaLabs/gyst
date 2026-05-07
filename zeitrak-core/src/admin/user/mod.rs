pub(crate) mod application;
pub(crate) mod domain;

pub use application::{
    UserRoot,
    commands::{UserCommand, UserCommandTrait},
    inputs::{CreateUserInput, UpdateUserSettingsInput},
    queries::{LoginQuery, LoginQueryTrait, UserQuery},
    rows::UserRow,
};
pub use domain::{
    aggregates::{User, UserId},
    events::UserEvent,
    interfaces::UserRepository,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("user not found")]
    NotFound,
    #[error("user already exists")]
    AlreadyExists,
}
