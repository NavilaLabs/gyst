use eventually::message::Message;
use serde::{Deserialize, Serialize};

use crate::workspace::activity;

fn default_color() -> String {
    "#6c6c76".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    /// A new activity was created.
    Created {
        id: activity::Id,
        name: String,
        #[serde(default = "default_color")]
        color: String,
        comment: Option<String>,
    },
    /// An activity was updated.
    Updated {
        name: String,
        #[serde(default = "default_color")]
        color: String,
        comment: Option<String>,
    },
    /// An activity was deleted.
    Deleted {},
}

impl Message for Event {
    fn name(&self) -> &'static str {
        match self {
            Self::Created { .. } => "ActivityCreated",
            Self::Updated { .. } => "ActivityUpdated",
            Self::Deleted { .. } => "ActivityDeleted",
        }
    }
}
