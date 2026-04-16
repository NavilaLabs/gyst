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

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn cancel(&mut self) -> Result<(), crate::Error> {
        self.record_that(TimesheetEvent::Cancelled {}.into())
            .map_err(|e| timesheet::DomainError::AggregateError(e).into())
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn update_time(
        &mut self,
        start_time: String,
        end_time: Option<String>,
        duration: Option<i32>,
    ) -> Result<(), crate::Error> {
        self.record_that(
            TimesheetEvent::TimeUpdated {
                start_time,
                end_time,
                duration,
            }
            .into(),
        )
        .map_err(|e| timesheet::DomainError::AggregateError(e).into())
    }
}

#[cfg(test)]
mod tests {
    use eventually::aggregate::{Aggregate, Root};

    use super::*;

    const TS_ID: &str = "019d0ce8-facb-7c90-b9d7-287ae4f17c91";
    const USER_ID: &str = "019d0ce8-facb-7c90-b9d7-287ae4f17c92";
    const ACT_ID: &str = "019d0ce8-facb-7c90-b9d7-287ae4f17c93";

    fn make_shell() -> TimesheetCommand {
        let id: TimesheetId = TS_ID.parse().unwrap();
        let user_id: UserId = USER_ID.parse().unwrap();
        let ts = Timesheet::apply(
            None,
            TimesheetEvent::Started {
                id,
                user_id,
                activity_id: None,
                start_time: "2024-01-01T09:00:00Z".to_string(),
                timezone: "UTC".to_string(),
            },
        )
        .expect("seed timesheet");
        Root::<Timesheet>::rehydrate_from_state(1, ts).into()
    }

    #[test]
    fn start_returns_root_with_applied_state() {
        let id: TimesheetId = "019d0ce8-facb-7c90-b9d7-287ae4f17d00".parse().unwrap();
        let user_id: UserId = USER_ID.parse().unwrap();

        let result = TimesheetCommand::start(
            id.clone(),
            user_id,
            None,
            "2024-01-02T09:00:00Z".to_string(),
            "UTC".to_string(),
        );

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.aggregate_id(), &id);
        assert_eq!(cmd.version(), 1);
    }

    #[test]
    fn stop_records_event_and_increments_version() {
        let mut cmd = make_shell();
        cmd.stop("2024-01-01T10:00:00Z".to_string(), 3600)
            .expect("stop must succeed");
        assert_eq!(cmd.version(), 2);
        assert_eq!(cmd.end_time(), Some("2024-01-01T10:00:00Z"));
        assert_eq!(cmd.duration(), Some(3600));
    }

    #[test]
    fn update_records_event_and_increments_version() {
        let mut cmd = make_shell();
        cmd.update(Some("pair session".to_string()))
            .expect("update must succeed");
        assert_eq!(cmd.version(), 2);
        assert_eq!(cmd.description(), Some("pair session"));
    }

    #[test]
    fn reassign_records_event_and_increments_version() {
        let mut cmd = make_shell();
        let act_id: ActivityId = ACT_ID.parse().unwrap();
        cmd.reassign(act_id.clone()).expect("reassign must succeed");
        assert_eq!(cmd.version(), 2);
        assert_eq!(cmd.activity_id(), Some(&act_id));
    }
}
