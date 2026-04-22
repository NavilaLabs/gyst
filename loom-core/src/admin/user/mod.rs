pub(crate) mod application;
pub(crate) mod domain;

pub use application::{
    Error as ApplicationError,
    commands::UserCommand,
    queries::{LoginQuery, UserQuery},
    rows::UserRow,
};
pub use domain::{
    Error as DomainError,
    aggregates::{User, UserId},
    events::UserEvent,
    interfaces::UserRepository,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0:?}")]
    ApplicationError(#[from] application::Error),
    #[error("{0:?}")]
    DomainError(#[from] domain::Error),
}
