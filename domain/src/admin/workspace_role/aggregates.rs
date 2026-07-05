use serde::{Deserialize, Serialize};

use crate::admin::workspace;

pub type Id = crate::AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregate {
    id: Id,
    workspace_id: workspace::Id,
    name: String,
    #[serde(default)]
    is_deleted: bool,
}

impl Aggregate {
    #[must_use]
    pub const fn id(&self) -> &Id {
        &self.id
    }

    #[must_use]
    pub const fn workspace_id(&self) -> &workspace::Id {
        &self.workspace_id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn is_deleted(&self) -> bool {
        self.is_deleted
    }
}
