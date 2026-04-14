pub(crate) mod application;
pub(crate) mod domain;

pub use application::{
    commands::TimesheetTagCommand,
    inputs::{CreateTimesheetTagInput, RenameTimesheetTagInput},
    views::TimesheetTagRow,
};
pub use domain::{
    Error as DomainError,
    aggregates::{TimesheetTag, TimesheetTagId},
    events::TimesheetTagEvent,
    interfaces::TimesheetTagRepository,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0:?}")]
    DomainError(#[from] domain::Error),
}
