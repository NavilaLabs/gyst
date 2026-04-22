pub mod aggregates;
pub mod events;
pub mod interfaces;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0:?}")]
    AggregateError(#[from] aggregates::Error),
}
