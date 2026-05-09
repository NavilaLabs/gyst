use eventually::message::Message;
use serde::{Deserialize, Serialize};

use crate::admin::permission::PermissionId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionEvent {
    Created { id: PermissionId, name: String },
}

impl Message for PermissionEvent {
    fn name(&self) -> &'static str {
        match self {
            Self::Created { .. } => "PermissionCreated",
        }
    }
}
