use eventually::message::Message;
use serde::{Deserialize, Serialize};

use crate::admin::permission;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Created { id: permission::Id, name: String },
}

impl Message for Event {
    fn name(&self) -> &'static str {
        match self {
            Self::Created { .. } => "PermissionCreated",
        }
    }
}
