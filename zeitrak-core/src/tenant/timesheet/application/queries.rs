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
    async fn recent_for_user(
        &self,
        user_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<TimesheetRow>, u64), Self::Error>;
    async fn running_for_user(&self, user_id: &str) -> Result<Option<TimesheetRow>, Self::Error>;
    async fn recent_for_workspace(
        &self,
        page: u32,
        page_size: u32,
        member_id: Option<&str>,
    ) -> Result<(Vec<TimesheetRow>, u64), Self::Error>;
    async fn stats_for_period(
        &self,
        member_id: Option<&str>,
        since_rfc3339: &str,
    ) -> Result<Vec<TimesheetRow>, Self::Error>;

    /// Returns a page of non-cancelled timesheets with optional date-range filtering.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn list_for_timeline(
        &self,
        page: u32,
        page_size: u32,
        from: Option<&str>,
        to: Option<&str>,
        member_id: Option<&str>,
    ) -> Result<(Vec<TimesheetRow>, u64), Self::Error>;

    /// Returns all completed timesheets in the optional date range for metrics.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn stats_for_timeline(
        &self,
        from: Option<&str>,
        to: Option<&str>,
        member_id: Option<&str>,
    ) -> Result<Vec<TimesheetRow>, Self::Error>;
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

    async fn recent_for_user(
        &self,
        user_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<TimesheetRow>, u64), Self::Error> {
        self.repository
            .recent_for_user(user_id, page, page_size)
            .await
    }

    async fn running_for_user(&self, user_id: &str) -> Result<Option<TimesheetRow>, Self::Error> {
        self.repository.running_for_user(user_id).await
    }

    async fn recent_for_workspace(
        &self,
        page: u32,
        page_size: u32,
        member_id: Option<&str>,
    ) -> Result<(Vec<TimesheetRow>, u64), Self::Error> {
        self.repository
            .recent_for_workspace(page, page_size, member_id)
            .await
    }

    async fn stats_for_period(
        &self,
        member_id: Option<&str>,
        since_rfc3339: &str,
    ) -> Result<Vec<TimesheetRow>, Self::Error> {
        self.repository
            .stats_for_period(member_id, since_rfc3339)
            .await
    }

    async fn list_for_timeline(
        &self,
        page: u32,
        page_size: u32,
        from: Option<&str>,
        to: Option<&str>,
        member_id: Option<&str>,
    ) -> Result<(Vec<TimesheetRow>, u64), Self::Error> {
        self.repository
            .list_for_timeline(page, page_size, from, to, member_id)
            .await
    }

    async fn stats_for_timeline(
        &self,
        from: Option<&str>,
        to: Option<&str>,
        member_id: Option<&str>,
    ) -> Result<Vec<TimesheetRow>, Self::Error> {
        self.repository
            .stats_for_timeline(from, to, member_id)
            .await
    }
}
