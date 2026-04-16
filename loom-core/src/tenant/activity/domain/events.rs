use eventually::message::Message;
use serde::{Deserialize, Serialize};

use crate::tenant::activity::ActivityId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityEvent {
    Created {
        id: ActivityId,
        name: String,
        comment: Option<String>,
    },
    Updated {
        name: String,
        comment: Option<String>,
    },
    Deleted {},
}

impl Message for ActivityEvent {
    fn name(&self) -> &'static str {
        match self {
            Self::Created { .. } => "ActivityCreated",
            Self::Updated { .. } => "ActivityUpdated",
            Self::Deleted { .. } => "ActivityDeleted",
        }
    }
}
