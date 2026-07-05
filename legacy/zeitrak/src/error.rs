//! Shared error types returned by zeitrak controllers.
//!
//! Presentation layers can downcast these to generate appropriate HTTP
//! responses (e.g. 422 Unprocessable Entity for `ValidationError`).

use thiserror::Error;

/// A domain-level input validation failure.
///
/// Controllers return this (via `anyhow::Error`) when the caller supplies
/// data that violates domain invariants (empty name, wrong format, etc.).
/// The presentation layer can downcast to this type and map it to a 422
/// response, while leaving all other errors as 500.
#[derive(Debug, Error)]
#[error("{0}")]
pub struct ValidationError(pub String);

impl ValidationError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

/// Run `validator` on `input` and return a [`ValidationError`] on failure.
///
/// Consolidates the three-line `.validate().map_err(|e| ValidationError::new(...))?`
/// boilerplate that would otherwise appear at every controller create/update call site.
pub fn validate<T: zeitrak_core::validation::Validate>(input: &T) -> anyhow::Result<()> {
    input
        .validate()
        .map_err(|e| ValidationError::new(zeitrak_core::validation::validation_summary(&e)))
        .map_err(Into::into)
}
