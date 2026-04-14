use eventually::aggregate;

use crate::tenant::timesheet::TimesheetId;
use crate::tenant::timesheet_tag::{
    self,
    domain::{
        aggregates::{TimesheetTag, TimesheetTagId},
        events::TimesheetTagEvent,
    },
};

#[eventually_macros::aggregate_root(TimesheetTag)]
pub struct TimesheetTagCommand;

impl TimesheetTagCommand {
    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn create(&self, id: TimesheetTagId, name: String) -> Result<Self, crate::Error> {
        Ok(aggregate::Root::<TimesheetTag>::record_new(
            TimesheetTagEvent::Created { id, name }.into(),
        )
        .map_err(timesheet_tag::DomainError::from)?
        .into())
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn rename(&mut self, name: String) -> Result<(), crate::Error> {
        self.record_that(TimesheetTagEvent::Renamed { name }.into())
            .map_err(|e| timesheet_tag::DomainError::AggregateError(e).into())
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn tag_timesheet(&mut self, timesheet_id: TimesheetId) -> Result<(), crate::Error> {
        self.record_that(TimesheetTagEvent::TimesheetTagged { timesheet_id }.into())
            .map_err(|e| timesheet_tag::DomainError::AggregateError(e).into())
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn untag_timesheet(&mut self, timesheet_id: TimesheetId) -> Result<(), crate::Error> {
        self.record_that(TimesheetTagEvent::TimesheetUntagged { timesheet_id }.into())
            .map_err(|e| timesheet_tag::DomainError::AggregateError(e).into())
    }
}
