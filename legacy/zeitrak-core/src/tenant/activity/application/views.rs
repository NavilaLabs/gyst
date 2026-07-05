use crate::tenant::activity::ActivityId;

#[derive(Debug, Clone)]
pub struct ActivityRow {
    id: ActivityId,
    name: String,
    color: String,
    comment: Option<String>,
}

impl ActivityRow {
    #[must_use]
    pub const fn new(id: ActivityId, name: String, color: String, comment: Option<String>) -> Self {
        Self {
            id,
            name,
            color,
            comment,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &ActivityId {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn color(&self) -> &str {
        &self.color
    }

    #[must_use]
    pub fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
}
