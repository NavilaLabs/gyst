use serde::{Deserialize, Serialize};

pub type Id = crate::AggregateId;

/// A permission to a certain action.
///
/// A permission that applies for the whole instance of zeitrak. Meaning it
/// is not only for a single workspace but is used for all of them. The
/// cannot be defined by any client but instead are defined by developers to
/// limit some kind of usecase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregate {
    id: Id,
    name: String,
}

impl Aggregate {
    /// The id of the permission.
    #[must_use]
    pub const fn id(&self) -> &Id {
        &self.id
    }

    /// The name of the permission.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}
