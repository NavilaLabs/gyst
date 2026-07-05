use chrono::{DateTime, Utc};

use crate::admin::{
    invitation::domain::aggregates::{InvitationId, InvitationStatus},
    workspace::WorkspaceId,
    workspace_role::WorkspaceRoleId,
};

#[derive(Debug, Clone)]
pub struct InvitationRow {
    id: InvitationId,
    pub workspace_id: WorkspaceId,
    pub workspace_name: Option<String>,
    pub email: String,
    pub workspace_role_id: WorkspaceRoleId,
    pub token: String,
    pub status: InvitationStatus,
    pub expires_at: DateTime<Utc>,
}

impl InvitationRow {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        id: InvitationId,
        workspace_id: WorkspaceId,
        workspace_name: Option<String>,
        email: String,
        workspace_role_id: WorkspaceRoleId,
        token: String,
        status: InvitationStatus,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            workspace_name,
            email,
            workspace_role_id,
            token,
            status,
            expires_at,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &InvitationId {
        &self.id
    }

    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }

    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }
}
