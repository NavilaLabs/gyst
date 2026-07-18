mod aggregates;
mod events;
mod repositories;

pub use aggregates::{Id, Aggregate};
pub use events::Event;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("timesheet already exists")]
    AlreadyExists,
    #[error("timesheet not found")]
    NotFound,
    #[error("timesheet already exported")]
    AlreadyExported,
    #[error("timesheet already cancelled")]
    AlreadyCancelled,
}
