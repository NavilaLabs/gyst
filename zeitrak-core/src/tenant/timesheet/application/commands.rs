use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate;

use crate::shared::AggregateId;
use crate::tenant::activity::ActivityId;
use crate::tenant::timesheet::{
    self,
    application::views::TimesheetRow,
    domain::{
        aggregates::{Timesheet, TimesheetId},
        events::{TimesheetEvent, UserId},
        interfaces::TimesheetRepository,
    },
};

pub trait TimesheetCommandTrait<T> {
    type Error: Debug + Sync + Send;

    /// # Errors
    ///
    /// Returns an error if the event cannot be applied to the aggregate.
    #[allow(clippy::too_many_arguments)]
    fn start(
        &self,
        id: TimesheetId,
        user_id: UserId,
        activity_id: Option<ActivityId>,
        start_time: String,
        timezone: String,
    ) -> Result<T, Self::Error>;

    /// # Errors
    ///
    /// Returns an error if the event cannot be applied to the aggregate.
    fn stop(&mut self, end_time: String, duration: i32) -> Result<(), Self::Error>;

    /// # Errors
    ///
    /// Returns an error if the event cannot be applied to the aggregate.
    fn update(&mut self, description: Option<String>) -> Result<(), Self::Error>;

    /// # Errors
    ///
    /// Returns an error if the event cannot be applied to the aggregate.
    fn reassign(&mut self, activity_id: ActivityId) -> Result<(), Self::Error>;

    /// # Errors
    ///
    /// Returns an error if the event cannot be applied to the aggregate.
    fn cancel(&mut self) -> Result<(), Self::Error>;

    /// # Errors
    ///
    /// Returns an error if the event cannot be applied to the aggregate.
    fn update_time(
        &mut self,
        start_time: String,
        end_time: Option<String>,
        duration: Option<i32>,
    ) -> Result<(), Self::Error>;
}

#[eventually_macros::aggregate_root(Timesheet)]
pub struct TimesheetCommand;

impl TimesheetCommandTrait<Self> for TimesheetCommand {
    type Error = timesheet::Error;

    #[allow(clippy::too_many_arguments)]
    fn start(
        &self,
        id: TimesheetId,
        user_id: UserId,
        activity_id: Option<ActivityId>,
        start_time: String,
        timezone: String,
    ) -> Result<Self, Self::Error> {
        Ok(aggregate::Root::<Timesheet>::record_new(
            TimesheetEvent::Started {
                id,
                user_id,
                activity_id,
                start_time,
                timezone,
            }
            .into(),
        )?
        .into())
    }

    fn stop(&mut self, end_time: String, duration: i32) -> Result<(), Self::Error> {
        self.record_that(TimesheetEvent::Stopped { end_time, duration }.into())
    }

    fn update(&mut self, description: Option<String>) -> Result<(), Self::Error> {
        self.record_that(TimesheetEvent::Updated { description }.into())
    }

    fn reassign(&mut self, activity_id: ActivityId) -> Result<(), Self::Error> {
        self.record_that(TimesheetEvent::Reassigned { activity_id }.into())
    }

    fn cancel(&mut self) -> Result<(), Self::Error> {
        self.record_that(TimesheetEvent::Cancelled {}.into())
    }

    fn update_time(
        &mut self,
        start_time: String,
        end_time: Option<String>,
        duration: Option<i32>,
    ) -> Result<(), Self::Error> {
        self.record_that(
            TimesheetEvent::TimeUpdated {
                start_time,
                end_time,
                duration,
            }
            .into(),
        )
    }
}

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
    ) -> Result<Self, timesheet::Error> {
        Ok(aggregate::Root::<Timesheet>::record_new(
            TimesheetEvent::Started {
                id,
                user_id,
                activity_id,
                start_time,
                timezone,
            }
            .into(),
        )?
        .into())
    }
}

#[async_trait]
pub trait TimesheetHandlerTrait<R> {
    type Error: Debug + Sync + Send;

    #[allow(clippy::too_many_arguments)]
    async fn start(
        &self,
        id: TimesheetId,
        user_id: AggregateId,
        activity_id: Option<ActivityId>,
        start_time: String,
        timezone: String,
        description: Option<String>,
    ) -> Result<TimesheetRow, Self::Error>;

    async fn stop(&self, id: TimesheetId, end_time: String, duration: i32)
        -> Result<(), Self::Error>;

    async fn update(
        &self,
        id: TimesheetId,
        description: Option<String>,
    ) -> Result<(), Self::Error>;

    async fn reassign(
        &self,
        id: TimesheetId,
        activity_id: ActivityId,
    ) -> Result<(), Self::Error>;

    async fn cancel(&self, id: TimesheetId) -> Result<(), Self::Error>;

    #[allow(clippy::too_many_arguments)]
    async fn create_manual(
        &self,
        id: TimesheetId,
        user_id: AggregateId,
        activity_id: Option<ActivityId>,
        start_time: String,
        end_time: String,
        duration: i32,
        description: Option<String>,
    ) -> Result<TimesheetRow, Self::Error>;

    async fn update_time(
        &self,
        id: TimesheetId,
        start_time: String,
        end_time: Option<String>,
        duration: Option<i32>,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub struct TimesheetHandler<Repo> {
    repository: Repo,
}

impl<Repo> TimesheetHandler<Repo> {
    pub const fn new(repository: Repo) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<Repo, R> TimesheetHandlerTrait<R> for TimesheetHandler<Repo>
where
    Repo: Debug + TimesheetRepository<R>,
{
    type Error = crate::Error<Repo, Timesheet, R>;

    async fn start(
        &self,
        id: TimesheetId,
        user_id: AggregateId,
        activity_id: Option<ActivityId>,
        start_time: String,
        timezone: String,
        description: Option<String>,
    ) -> Result<TimesheetRow, Self::Error> {
        let mut root: TimesheetCommand = aggregate::Root::<Timesheet>::record_new(
            TimesheetEvent::Started {
                id: id.clone(),
                user_id: user_id.clone(),
                activity_id: activity_id.clone(),
                start_time: start_time.clone(),
                timezone: timezone.clone(),
            }
            .into(),
        )?
        .into();
        if let Some(ref desc) = description {
            root.update(Some(desc.clone()))?;
        }
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))?;
        Ok(TimesheetRow::new(
            id,
            user_id,
            activity_id,
            start_time,
            None,
            None,
            description,
            timezone,
        ))
    }

    async fn stop(
        &self,
        id: TimesheetId,
        end_time: String,
        duration: i32,
    ) -> Result<(), Self::Error> {
        let mut root: TimesheetCommand = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();
        root.stop(end_time, duration)?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
    }

    async fn update(
        &self,
        id: TimesheetId,
        description: Option<String>,
    ) -> Result<(), Self::Error> {
        let mut root: TimesheetCommand = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();
        root.update(description)?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
    }

    async fn reassign(
        &self,
        id: TimesheetId,
        activity_id: ActivityId,
    ) -> Result<(), Self::Error> {
        let mut root: TimesheetCommand = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();
        root.reassign(activity_id)?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
    }

    async fn cancel(&self, id: TimesheetId) -> Result<(), Self::Error> {
        let mut root: TimesheetCommand = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();
        root.cancel()?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
    }

    async fn create_manual(
        &self,
        id: TimesheetId,
        user_id: AggregateId,
        activity_id: Option<ActivityId>,
        start_time: String,
        end_time: String,
        duration: i32,
        description: Option<String>,
    ) -> Result<TimesheetRow, Self::Error> {
        let mut root: TimesheetCommand = aggregate::Root::<Timesheet>::record_new(
            TimesheetEvent::Started {
                id: id.clone(),
                user_id: user_id.clone(),
                activity_id: activity_id.clone(),
                start_time: start_time.clone(),
                timezone: "UTC".to_string(),
            }
            .into(),
        )?
        .into();
        root.stop(end_time.clone(), duration)?;
        if let Some(ref desc) = description {
            root.update(Some(desc.clone()))?;
        }
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))?;
        Ok(TimesheetRow::new(
            id,
            user_id,
            activity_id,
            start_time,
            Some(end_time),
            Some(duration),
            description,
            "UTC".to_string(),
        ))
    }

    async fn update_time(
        &self,
        id: TimesheetId,
        start_time: String,
        end_time: Option<String>,
        duration: Option<i32>,
    ) -> Result<(), Self::Error> {
        let mut root: TimesheetCommand = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();
        root.update_time(start_time, end_time, duration)?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
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

    #[test]
    fn apply_stopped_sets_end_time_and_duration() {
        let mut cmd = make_shell();
        cmd.stop("2024-01-01T17:00:00Z".to_string(), 7200)
            .expect("stop must succeed");
        assert_eq!(cmd.end_time(), Some("2024-01-01T17:00:00Z"));
        assert_eq!(cmd.duration(), Some(7200));
    }

    #[test]
    fn apply_time_updated_overwrites_all_time_fields() {
        let mut cmd = make_shell();
        cmd.update_time(
            "2024-01-01T10:00:00Z".to_string(),
            Some("2024-01-01T12:00:00Z".to_string()),
            Some(7200),
        )
        .expect("update_time must succeed");
        assert_eq!(cmd.start_time(), "2024-01-01T10:00:00Z");
        assert_eq!(cmd.end_time(), Some("2024-01-01T12:00:00Z"));
        assert_eq!(cmd.duration(), Some(7200));
    }
}
