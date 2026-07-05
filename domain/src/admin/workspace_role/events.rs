use eventually::message::Message;
use serde::{Deserialize, Serialize};

use crate::admin::{permission, workspace, workspace_role};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Created {
        id: workspace_role::Id,
        workspace_id: workspace::Id,
        name: String,
    },
    PermissionGranted {
        permission_id: permission::Id,
    },
    PermissionRevoked {
        permission_id: permission::Id,
    },
    Renamed {
        name: String,
    },
    Deleted,
}

impl Message for Event {
    fn name(&self) -> &'static str {
        match self {
            Self::Created { .. } => "WorkspaceRoleCreated",
            Self::PermissionGranted { .. } => "WorkspaceRolePermissionGranted",
            Self::PermissionRevoked { .. } => "WorkspaceRolePermissionRevoked",
            Self::Renamed { .. } => "WorkspaceRoleRenamed",
            Self::Deleted => "WorkspaceRoleDeleted",
        }
    }
}
