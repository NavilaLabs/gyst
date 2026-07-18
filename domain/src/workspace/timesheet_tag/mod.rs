mod aggregates;
mod events;
mod repositories;

pub use aggregates::{Aggregate, Id};
pub use events::Event;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("timesheet tag already exists")]
    AlreadyExists,
    #[error("timesheet tag not found")]
    NotFound,
}
