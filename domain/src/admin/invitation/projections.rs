use chrono::{DateTime, Utc};

use crate::admin::{invitation, workspace, workspace_role};

#[derive(Debug, Clone)]
pub struct Projection {
    id: invitation::Id,
    workspace_id: workspace::Id,
    workspace_role_id: workspace_role::Id,
    email: String,
    token: String,
    status: invitation::Status,
    expires_at: DateTime<Utc>,
}

impl Projection {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        id: invitation::Id,
        workspace_id: workspace::Id,
        workspace_role_id: workspace_role::Id,
        email: String,
        token: String,
        status: invitation::Status,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            workspace_role_id,
            email,
            token,
            status,
            expires_at,
        }
    }
}

impl Projection {
    #[must_use]
    pub const fn id(&self) -> &invitation::Id {
        &self.id
    }

    #[must_use]
    pub const fn workspace_id(&self) -> &workspace::Id {
        &self.workspace_id
    }

    #[must_use]
    pub const fn workspace_role_id(&self) -> &workspace::Id {
        &self.workspace_role_id
    }

    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }

    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    #[must_use]
    pub const fn status(&self) -> &invitation::Status {
        &self.status
    }

    #[must_use]
    pub const fn expires_at(&self) -> &DateTime<Utc> {
        &self.expires_at
    }
}
