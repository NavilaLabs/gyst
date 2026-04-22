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
    pub fn create(id: TimesheetTagId, name: String) -> Result<Self, timesheet_tag::DomainError> {
        Ok(aggregate::Root::<TimesheetTag>::record_new(
            TimesheetTagEvent::Created { id, name }.into(),
        )
        .map_err(timesheet_tag::DomainError::AggregateError)?
        .into())
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn rename(&mut self, name: String) -> Result<(), timesheet_tag::DomainError> {
        self.record_that(TimesheetTagEvent::Renamed { name }.into())
            .map_err(timesheet_tag::DomainError::AggregateError)
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn tag_timesheet(&mut self, timesheet_id: TimesheetId) -> Result<(), timesheet_tag::DomainError> {
        self.record_that(TimesheetTagEvent::TimesheetTagged { timesheet_id }.into())
            .map_err(timesheet_tag::DomainError::AggregateError)
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn untag_timesheet(&mut self, timesheet_id: TimesheetId) -> Result<(), timesheet_tag::DomainError> {
        self.record_that(TimesheetTagEvent::TimesheetUntagged { timesheet_id }.into())
            .map_err(timesheet_tag::DomainError::AggregateError)
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn delete(&mut self) -> Result<(), timesheet_tag::DomainError> {
        self.record_that(TimesheetTagEvent::Deleted {}.into())
            .map_err(timesheet_tag::DomainError::AggregateError)
    }
}

#[cfg(test)]
mod tests {
    use eventually::aggregate::{Aggregate, Root};

    use super::*;

    fn test_id() -> TimesheetTagId {
        "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
            .parse()
            .expect("valid UUID")
    }

    fn make_shell(id: TimesheetTagId) -> TimesheetTagCommand {
        let tag = TimesheetTag::apply(
            None,
            TimesheetTagEvent::Created {
                id,
                name: "seed".to_string(),
            },
        )
        .expect("seed tag");
        Root::<TimesheetTag>::rehydrate_from_state(1, tag).into()
    }

    #[test]
    fn create_returns_root_with_applied_state() {
        let id: TimesheetTagId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92".parse().unwrap();

        let result = TimesheetTagCommand::create(id.clone(), "backend".to_string());

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.aggregate_id(), &id);
        assert_eq!(cmd.name(), "backend");
        assert_eq!(cmd.version(), 1);
    }

    #[test]
    fn rename_records_event_and_increments_version() {
        let mut cmd = make_shell(test_id());
        cmd.rename("frontend".to_string())
            .expect("rename must succeed");
        assert_eq!(cmd.version(), 2);
        assert_eq!(cmd.name(), "frontend");
    }

    #[test]
    fn tag_timesheet_records_event_and_increments_version() {
        let mut cmd = make_shell(test_id());
        let ts_id: TimesheetId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92".parse().unwrap();
        cmd.tag_timesheet(ts_id)
            .expect("tag_timesheet must succeed");
        assert_eq!(cmd.version(), 2);
    }

    #[test]
    fn untag_timesheet_records_event_and_increments_version() {
        let mut cmd = make_shell(test_id());
        let ts_id: TimesheetId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92".parse().unwrap();
        cmd.tag_timesheet(ts_id.clone()).unwrap();
        cmd.untag_timesheet(ts_id)
            .expect("untag_timesheet must succeed");
        assert_eq!(cmd.version(), 3);
    }
}
