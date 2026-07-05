use crate::admin::workspace::WorkspaceId;

/// Read model for a workspace member with their assigned roles and direct permissions.
#[derive(Debug, Clone)]
pub struct MemberRow {
    pub user_id: String,
    pub email: String,
    pub name: String,
    pub role_ids: Vec<String>,
    pub permission_ids: Vec<String>,
}

impl MemberRow {
    #[must_use]
    pub const fn new(
        user_id: String,
        email: String,
        name: String,
        role_ids: Vec<String>,
        permission_ids: Vec<String>,
    ) -> Self {
        Self {
            user_id,
            email,
            name,
            role_ids,
            permission_ids,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkspaceRow {
    id: WorkspaceId,
    name: Option<String>,
    pub timezone: String,
    pub date_format: String,
    pub currency: String,
    pub week_start: String,
}

impl WorkspaceRow {
    #[must_use]
    pub fn new(id: WorkspaceId, name: Option<String>) -> Self {
        Self {
            id,
            name,
            timezone: "Europe/Berlin".to_string(),
            date_format: "%Y-%m-%d".to_string(),
            currency: "EUR".to_string(),
            week_start: "monday".to_string(),
        }
    }

    #[must_use]
    pub const fn new_with_settings(
        id: WorkspaceId,
        name: Option<String>,
        timezone: String,
        date_format: String,
        currency: String,
        week_start: String,
    ) -> Self {
        Self {
            id,
            name,
            timezone,
            date_format,
            currency,
            week_start,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &WorkspaceId {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}
