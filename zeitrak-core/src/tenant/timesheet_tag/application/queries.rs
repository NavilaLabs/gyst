use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate::Root;

use crate::tenant::timesheet_tag::{
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
    R: Debug + Send + Sync,
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
}
