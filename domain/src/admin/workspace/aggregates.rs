use serde::{Deserialize, Serialize};

pub type Id = crate::AggregateId;

/// An aggregate representing a workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregate {
    /// The `id` of a workspace.
    id: Id,
    /// The `name` of a workspace.
    name: String,
}

impl Aggregate {
    /// Get the `id` of a workspace.
    #[must_use]
    pub const fn id(&self) -> &Id {
        &self.id
    }

    /// Get the `name` of a workspace.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}
