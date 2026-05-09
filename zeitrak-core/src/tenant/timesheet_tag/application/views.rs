use crate::tenant::timesheet_tag::TimesheetTagId;

#[derive(Debug, Clone)]
pub struct TimesheetTagRow {
    id: TimesheetTagId,
    name: String,
}

impl TimesheetTagRow {
    #[must_use]
    pub const fn new(id: TimesheetTagId, name: String) -> Self {
        Self { id, name }
    }

    #[must_use]
    pub const fn id(&self) -> &TimesheetTagId {
        &self.id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}
