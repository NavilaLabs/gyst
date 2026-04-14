use crate::tenant::activity::ActivityId;

#[derive(Debug, Clone)]
pub struct ActivityView {
    id: ActivityId,
    name: String,
    comment: Option<String>,
}

impl ActivityView {
    #[must_use]
    pub const fn new(id: ActivityId, name: String, comment: Option<String>) -> Self {
        Self { id, name, comment }
    }

    #[must_use]
    pub const fn get_id(&self) -> &ActivityId {
        &self.id
    }

    #[must_use]
    pub fn get_name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn get_comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
}
