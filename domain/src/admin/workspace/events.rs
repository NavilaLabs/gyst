use eventually::message::Message;
use serde::{Deserialize, Serialize};

use crate::admin::{permission, user, workspace, workspace_role};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Created {
        id: workspace::Id,
        name: String,
    },
    UserRoleAssigned {
        user_id: user::Id,
        workspace_role_id: workspace_role::Id,
    },
    UserRoleRevoked {
        user_id: user::Id,
        workspace_role_id: workspace_role::Id,
    },
    UserPermissionGranted {
        user_id: user::Id,
        permission_id: permission::Id,
    },
    UserPermissionRevoked {
        user_id: user::Id,
        permission_id: permission::Id,
    },
    SettingsUpdated {
        name: String,
    },
    UserRemoved {
        user_id: user::Id,
    },
}

impl Message for Event {
    fn name(&self) -> &'static str {
        match self {
            Self::Created { .. } => "WorkspaceCreated",
            Self::UserRoleAssigned { .. } => "WorkspaceUserRoleAssigned",
            Self::UserRoleRevoked { .. } => "WorkspaceUserRoleRevoked",
            Self::UserPermissionGranted { .. } => "WorkspaceUserPermissionGranted",
            Self::UserPermissionRevoked { .. } => "WorkspaceUserPermissionRevoked",
            Self::SettingsUpdated { .. } => "WorkspaceSettingsUpdated",
            Self::UserRemoved { .. } => "WorkspaceUserRemoved",
        }
    }
}
