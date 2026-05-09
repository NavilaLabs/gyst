use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate::Root;

use crate::tenant::timesheet::{
    application::views::TimesheetRow,
    domain::aggregates::{Timesheet, TimesheetId},
    domain::interfaces::TimesheetRepository,
};

#[async_trait]
pub trait TimesheetQueryTrait<R> {
    type Error: Debug + Send + Sync;

    async fn find_by_id(&self, id: TimesheetId) -> Result<Option<Root<Timesheet>>, Self::Error>;
    async fn find_all(&self) -> Result<Vec<Root<Timesheet>>, Self::Error>;
    async fn recent_for_user(&self, user_id: &str) -> Result<Vec<TimesheetRow>, Self::Error>;
    async fn running_for_user(&self, user_id: &str) -> Result<Option<TimesheetRow>, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct TimesheetQuery<Repo> {
    repository: Repo,
}

impl<Repo> TimesheetQuery<Repo> {
    pub const fn new(repository: Repo) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<Repo, R> TimesheetQueryTrait<R> for TimesheetQuery<Repo>
where
    Repo: Debug + Send + Sync + TimesheetRepository<R>,
{
    type Error = <Repo as TimesheetRepository<R>>::Error;

    async fn find_by_id(&self, id: TimesheetId) -> Result<Option<Root<Timesheet>>, Self::Error> {
        self.repository.find(id).await.map_err(Into::into)
    }

    async fn find_all(&self) -> Result<Vec<Root<Timesheet>>, Self::Error> {
        self.repository.all().await.map_err(Into::into)
    }

    async fn recent_for_user(&self, user_id: &str) -> Result<Vec<TimesheetRow>, Self::Error> {
        self.repository.recent_for_user(user_id).await
    }

    async fn running_for_user(&self, user_id: &str) -> Result<Option<TimesheetRow>, Self::Error> {
        self.repository.running_for_user(user_id).await
    }
}
