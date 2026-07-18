mod aggregates;
mod events;
mod repositories;

pub use aggregates::{Aggregate, Id};
pub use events::Event;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("workspace role already exists")]
    AlreadyExists,
    #[error("workspace role not found")]
    NotFound,
}
