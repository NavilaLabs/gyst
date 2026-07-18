use eventually::message::Message;
use serde::{Deserialize, Serialize};

use crate::admin::{permission, user, workspace, workspace_role};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    /// A user created a new workspace.
    Created {
        id: workspace::Id,
        name: String,
        is_deleted: bool,
    },
    /// A workspace admin updated the workspace settings.
    SettingsUpdated {
        name: String,
    },
    /// A workspace admin assigned a member with a role.
    UserRoleAssigned {
        user_id: user::Id,
        workspace_role_id: workspace_role::Id,
    },
    /// A workspace admin revoked a role from a member.
    UserRoleRevoked {
        user_id: user::Id,
        workspace_role_id: workspace_role::Id,
    },
    /// A workspace admin directly granted a permission for a member.
    UserPermissionGranted {
        user_id: user::Id,
        permission_id: permission::Id,
    },
    /// A workspace admin revoked a permission from a member.
    ///
    /// If a member is still assigned role which includes the permission
    /// this does not cancel out the role.
    UserPermissionRevoked {
        user_id: user::Id,
        permission_id: permission::Id,
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
