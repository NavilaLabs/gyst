pub(crate) mod application;
pub(crate) mod domain;

pub use application::{
    commands::ActivityCommand,
    inputs::{CreateActivityInput, UpdateActivityInput},
    views::ActivityRow,
};
pub use domain::{
    Error as DomainError,
    aggregates::{Activity, ActivityId},
    events::ActivityEvent,
    interfaces::ActivityRepository,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0:?}")]
    DomainError(#[from] domain::Error),
}
