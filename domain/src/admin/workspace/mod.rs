mod aggregates;
mod events;
mod repositories;

pub use aggregates::{Aggregate, Id};
pub use events::Event;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("workspace already exists")]
    AlreadyExists,
    #[error("workspace not found")]
    NotFound,
}
