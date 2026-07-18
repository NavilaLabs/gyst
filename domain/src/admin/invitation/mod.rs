mod aggregate;
mod event;
mod projections;
mod repositories;

pub use aggregate::{Aggregate, Id, Status};
pub use event::Event;
pub use projections::Projection;
pub use repositories::Repository;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invitation already exists")]
    AlreadyExists,
}
