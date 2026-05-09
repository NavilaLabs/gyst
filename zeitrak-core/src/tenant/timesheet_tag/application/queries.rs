use std::collections::HashMap;
use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate::Root;

use crate::tenant::timesheet_tag::{
    application::views::TimesheetTagRow,
    domain::aggregates::{TimesheetTag, TimesheetTagId},
    domain::interfaces::TimesheetTagRepository,
};

#[async_trait]
pub trait TimesheetTagQueryTrait<R> {
    type Error: Debug + Send + Sync;

    async fn find_by_id(
        &self,
        id: TimesheetTagId,
    ) -> Result<Option<Root<TimesheetTag>>, Self::Error>;
    async fn find_all(&self) -> Result<Vec<Root<TimesheetTag>>, Self::Error>;
    async fn list_all(&self) -> Result<Vec<TimesheetTagRow>, Self::Error>;
    async fn for_timesheet(&self, timesheet_id: &str) -> Result<Vec<TimesheetTagRow>, Self::Error>;
    async fn for_timesheets_batch(
        &self,
        timesheet_ids: &[&str],
    ) -> Result<HashMap<String, Vec<TimesheetTagRow>>, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct TimesheetTagQuery<Repo> {
    repository: Repo,
}

impl<Repo> TimesheetTagQuery<Repo> {
    pub const fn new(repository: Repo) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<Repo, R> TimesheetTagQueryTrait<R> for TimesheetTagQuery<Repo>
where
    Repo: Debug + Send + Sync + TimesheetTagRepository<R>,
{
    type Error = <Repo as TimesheetTagRepository<R>>::Error;

    async fn find_by_id(
        &self,
        id: TimesheetTagId,
    ) -> Result<Option<Root<TimesheetTag>>, Self::Error> {
        self.repository.find(id).await.map_err(Into::into)
    }

    async fn find_all(&self) -> Result<Vec<Root<TimesheetTag>>, Self::Error> {
        self.repository.all().await.map_err(Into::into)
    }

    async fn list_all(&self) -> Result<Vec<TimesheetTagRow>, Self::Error> {
        self.repository.list_all().await
    }

    async fn for_timesheet(&self, timesheet_id: &str) -> Result<Vec<TimesheetTagRow>, Self::Error> {
        self.repository.for_timesheet(timesheet_id).await
    }

    async fn for_timesheets_batch(
        &self,
        timesheet_ids: &[&str],
    ) -> Result<HashMap<String, Vec<TimesheetTagRow>>, Self::Error> {
        self.repository.for_timesheets_batch(timesheet_ids).await
    }
}
