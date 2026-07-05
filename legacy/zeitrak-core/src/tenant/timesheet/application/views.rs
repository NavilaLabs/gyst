use crate::shared::AggregateId;
use crate::tenant::activity::ActivityId;
use crate::tenant::timesheet::TimesheetId;

pub type UserId = AggregateId;

#[derive(Debug, Clone)]
pub struct TimesheetRow {
    id: TimesheetId,
    user_id: UserId,
    activity_id: Option<ActivityId>,
    start_time: String,
    end_time: Option<String>,
    duration: Option<i32>,
    description: Option<String>,
    timezone: String,
}

impl TimesheetRow {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        id: TimesheetId,
        user_id: UserId,
        activity_id: Option<ActivityId>,
        start_time: String,
        end_time: Option<String>,
        duration: Option<i32>,
        description: Option<String>,
        timezone: String,
    ) -> Self {
        Self {
            id,
            user_id,
            activity_id,
            start_time,
            end_time,
            duration,
            description,
            timezone,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &TimesheetId {
        &self.id
    }

    #[must_use]
    pub const fn user_id(&self) -> &UserId {
        &self.user_id
    }

    #[must_use]
    pub const fn activity_id(&self) -> Option<&ActivityId> {
        self.activity_id.as_ref()
    }

    #[must_use]
    pub fn start_time(&self) -> &str {
        &self.start_time
    }

    #[must_use]
    pub fn end_time(&self) -> Option<&str> {
        self.end_time.as_deref()
    }

    #[must_use]
    pub const fn duration(&self) -> Option<i32> {
        self.duration
    }

    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    #[must_use]
    pub fn timezone(&self) -> &str {
        &self.timezone
    }
}
