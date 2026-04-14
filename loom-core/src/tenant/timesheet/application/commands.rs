use eventually::aggregate;

use crate::tenant::activity::ActivityId;
use crate::tenant::timesheet::{
    self,
    domain::{
        aggregates::{Timesheet, TimesheetId},
        events::{TimesheetEvent, UserId},
    },
};

#[eventually_macros::aggregate_root(Timesheet)]
pub struct TimesheetCommand;

impl TimesheetCommand {
    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        &self,
        id: TimesheetId,
        user_id: UserId,
        activity_id: Option<ActivityId>,
        start_time: String,
        timezone: String,
    ) -> Result<Self, crate::Error> {
        Ok(aggregate::Root::<Timesheet>::record_new(
            TimesheetEvent::Started {
                id,
                user_id,
                activity_id,
                start_time,
                timezone,
            }
            .into(),
        )
        .map_err(timesheet::DomainError::from)?
        .into())
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    #[allow(clippy::too_many_arguments)]
    pub fn stop(&mut self, end_time: String, duration: i32) -> Result<(), crate::Error> {
        self.record_that(TimesheetEvent::Stopped { end_time, duration }.into())
            .map_err(|e| timesheet::DomainError::AggregateError(e).into())
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn update(&mut self, description: Option<String>) -> Result<(), crate::Error> {
        self.record_that(TimesheetEvent::Updated { description }.into())
            .map_err(|e| timesheet::DomainError::AggregateError(e).into())
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn reassign(&mut self, activity_id: ActivityId) -> Result<(), crate::Error> {
        self.record_that(TimesheetEvent::Reassigned { activity_id }.into())
            .map_err(|e| timesheet::DomainError::AggregateError(e).into())
    }
}
