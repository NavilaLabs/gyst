use crate::admin::{workspace::WorkspaceId, workspace_role::WorkspaceRoleId};

/// Read model for a workspace role enriched with its permission IDs and names.
#[derive(Debug, Clone)]
pub struct WorkspaceRoleWithPermissionsRow {
    pub id: String,
    pub workspace_id: String,
    pub name: Option<String>,
    pub permission_ids: Vec<String>,
    pub permission_names: Vec<String>,
}

impl WorkspaceRoleWithPermissionsRow {
    #[must_use]
    pub const fn new(
        id: String,
        workspace_id: String,
        name: Option<String>,
        permission_ids: Vec<String>,
        permission_names: Vec<String>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            name,
            permission_ids,
            permission_names,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkspaceRoleRow {
    id: WorkspaceRoleId,
    workspace_id: WorkspaceId,
    name: Option<String>,
}

impl WorkspaceRoleRow {
    #[must_use]
    pub const fn new(id: WorkspaceRoleId, workspace_id: WorkspaceId, name: Option<String>) -> Self {
        Self {
            id,
            workspace_id,
            name,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &WorkspaceRoleId {
        &self.id
    }

    #[must_use]
    pub const fn workspace_id(&self) -> &WorkspaceId {
        &self.workspace_id
    }

    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}
