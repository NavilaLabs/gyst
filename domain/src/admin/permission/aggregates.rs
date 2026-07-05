use serde::{Deserialize, Serialize};

use crate::admin::permission;

pub type Id = crate::AggregateId;

/// An aggregate repesenting a permission.
///
/// A permission that applies for the whole instance of zeitrak. Meaning it
/// is not only for a single workspace but is used for all of them. The
/// cannot be defined by any client but instead are defined by developers to
/// limit some kind of usecase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregate {
    /// The `id` of a permission.
    id: Id,
    /// The `name` of a permission.
    name: String,
}

impl Aggregate {
    /// Get the `id` of a permission.
    #[must_use]
    pub const fn id(&self) -> &Id {
        &self.id
    }

    /// Get the `name` of a permission.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl eventually::aggregate::Aggregate for Aggregate {
    type Id = Id;
    type Event = permission::Event;
    type Error = permission::Error;

    fn type_name() -> &'static str {
        "permission"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match (state, event) {
            (None, permission::Event::Created { id, name }) => Ok(Self { id, name }),
            (Some(_), permission::Event::Created { .. }) => Err(permission::Error::AlreadyExists),
        }
    }
}
