mod aggregates;
mod events;
mod repositories;

pub use aggregates::{Aggregate, Id};
pub use events::Event;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("activity already exists")]
    AlreadyExists,
    #[error("activity not found")]
    NotFound,
}
