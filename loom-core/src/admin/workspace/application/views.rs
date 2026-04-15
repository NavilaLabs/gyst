use crate::admin::workspace::WorkspaceId;

#[derive(Debug, Clone)]
pub struct WorkspaceView {
    id: WorkspaceId,
    name: Option<String>,
    pub timezone: String,
    pub date_format: String,
    pub currency: String,
    pub week_start: String,
}

impl WorkspaceView {
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
