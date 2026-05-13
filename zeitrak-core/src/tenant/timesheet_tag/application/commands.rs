use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate;

use crate::tenant::timesheet::TimesheetId;
use crate::tenant::timesheet_tag::{
    self,
    application::views::TimesheetTagRow,
    domain::{
        aggregates::{TimesheetTag, TimesheetTagId},
        events::TimesheetTagEvent,
        interfaces::TimesheetTagRepository,
    },
};

pub trait TimesheetTagCommandTrait<T> {
    type Error: Debug + Sync + Send;

    /// # Errors
    ///
    /// Returns an error if the event cannot be applied to the aggregate.
    fn create(&self, id: TimesheetTagId, name: String) -> Result<T, Self::Error>;

    /// # Errors
    ///
    /// Returns an error if the event cannot be applied to the aggregate.
    fn rename(&mut self, name: String) -> Result<(), Self::Error>;

    /// # Errors
    ///
    /// Returns an error if the event cannot be applied to the aggregate.
    fn tag_timesheet(&mut self, timesheet_id: TimesheetId) -> Result<(), Self::Error>;

    /// # Errors
    ///
    /// Returns an error if the event cannot be applied to the aggregate.
    fn untag_timesheet(&mut self, timesheet_id: TimesheetId) -> Result<(), Self::Error>;

    /// # Errors
    ///
    /// Returns an error if the event cannot be applied to the aggregate.
    fn delete(&mut self) -> Result<(), Self::Error>;
}

#[eventually_macros::aggregate_root(TimesheetTag)]
pub struct TimesheetTagCommand;

impl TimesheetTagCommandTrait<Self> for TimesheetTagCommand {
    type Error = timesheet_tag::Error;

    fn create(&self, id: TimesheetTagId, name: String) -> Result<Self, Self::Error> {
        Ok(aggregate::Root::<TimesheetTag>::record_new(
            TimesheetTagEvent::Created { id, name }.into(),
        )?
        .into())
    }

    fn rename(&mut self, name: String) -> Result<(), Self::Error> {
        self.record_that(TimesheetTagEvent::Renamed { name }.into())
    }

    fn tag_timesheet(&mut self, timesheet_id: TimesheetId) -> Result<(), Self::Error> {
        self.record_that(TimesheetTagEvent::TimesheetTagged { timesheet_id }.into())
    }

    fn untag_timesheet(&mut self, timesheet_id: TimesheetId) -> Result<(), Self::Error> {
        self.record_that(TimesheetTagEvent::TimesheetUntagged { timesheet_id }.into())
    }

    fn delete(&mut self) -> Result<(), Self::Error> {
        self.record_that(TimesheetTagEvent::Deleted {}.into())
    }
}

impl TimesheetTagCommand {
    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn create(id: TimesheetTagId, name: String) -> Result<Self, timesheet_tag::Error> {
        Ok(aggregate::Root::<TimesheetTag>::record_new(
            TimesheetTagEvent::Created { id, name }.into(),
        )?
        .into())
    }
}

#[async_trait]
pub trait TimesheetTagHandlerTrait<R> {
    type Error: Debug + Sync + Send;

    async fn create(
        &self,
        id: TimesheetTagId,
        name: String,
    ) -> Result<TimesheetTagRow, Self::Error>;

    async fn rename(&self, id: TimesheetTagId, name: String) -> Result<(), Self::Error>;

    async fn tag_timesheet(
        &self,
        tag_id: TimesheetTagId,
        timesheet_id: TimesheetId,
    ) -> Result<(), Self::Error>;

    async fn untag_timesheet(
        &self,
        tag_id: TimesheetTagId,
        timesheet_id: TimesheetId,
    ) -> Result<(), Self::Error>;

    async fn delete(&self, id: TimesheetTagId) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub struct TimesheetTagHandler<Repo> {
    repository: Repo,
}

impl<Repo> TimesheetTagHandler<Repo> {
    pub const fn new(repository: Repo) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<Repo, R> TimesheetTagHandlerTrait<R> for TimesheetTagHandler<Repo>
where
    Repo: Debug + TimesheetTagRepository<R>,
{
    type Error = crate::Error<Repo, TimesheetTag, R>;

    async fn create(
        &self,
        id: TimesheetTagId,
        name: String,
    ) -> Result<TimesheetTagRow, Self::Error> {
        let mut root = aggregate::Root::<TimesheetTag>::record_new(
            TimesheetTagEvent::Created {
                id: id.clone(),
                name: name.clone(),
            }
            .into(),
        )?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))?;
        Ok(TimesheetTagRow::new(id, name))
    }

    async fn rename(&self, id: TimesheetTagId, name: String) -> Result<(), Self::Error> {
        let mut root: TimesheetTagCommand = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();
        root.rename(name)?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
    }

    async fn tag_timesheet(
        &self,
        tag_id: TimesheetTagId,
        timesheet_id: TimesheetId,
    ) -> Result<(), Self::Error> {
        let mut root: TimesheetTagCommand = self
            .repository
            .get(&tag_id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();
        root.tag_timesheet(timesheet_id)?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
    }

    async fn untag_timesheet(
        &self,
        tag_id: TimesheetTagId,
        timesheet_id: TimesheetId,
    ) -> Result<(), Self::Error> {
        let mut root: TimesheetTagCommand = self
            .repository
            .get(&tag_id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();
        root.untag_timesheet(timesheet_id)?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
    }

    async fn delete(&self, id: TimesheetTagId) -> Result<(), Self::Error> {
        let mut root: TimesheetTagCommand = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();
        root.delete()?;
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
