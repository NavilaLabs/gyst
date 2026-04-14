use crate::tenant::timesheet_tag::TimesheetTagId;

#[derive(Debug, Clone)]
pub struct TimesheetTagView {
    id: TimesheetTagId,
    name: String,
}

impl TimesheetTagView {
    #[must_use]
    pub const fn new(id: TimesheetTagId, name: String) -> Self {
        Self { id, name }
    }

    #[must_use]
    pub const fn get_id(&self) -> &TimesheetTagId {
        &self.id
    }
    #[must_use]
    pub fn get_name(&self) -> &str {
        &self.name
    }
}
