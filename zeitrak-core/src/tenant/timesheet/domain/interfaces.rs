use std::fmt::Debug;

use crate::{
    shared::repositories::{ReadRepository, Repository, WriteRepository},
    tenant::timesheet::{application::views::TimesheetRow, domain::aggregates::Timesheet},
};
use async_trait::async_trait;

#[async_trait]
pub trait TimesheetRepository<R>: Repository<Timesheet, R> + Send + Sync {
    type Error: Debug
        + Send
        + Sync
        + From<<Self as ReadRepository<Timesheet, R>>::Error>
        + From<<Self as WriteRepository<Timesheet>>::Error>;

    /// Returns the most-recent 50 non-cancelled timesheets for a user, newest first.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn recent_for_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<TimesheetRow>, <Self as TimesheetRepository<R>>::Error>;

    /// Returns the running timesheet for a user (`end_time` IS NULL), if any.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn running_for_user(
        &self,
        user_id: &str,
    ) -> Result<Option<TimesheetRow>, <Self as TimesheetRepository<R>>::Error>;

    /// Returns the most-recent 50 non-cancelled timesheets across all users in
    /// the workspace, newest first.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn recent_for_workspace(
        &self,
    ) -> Result<Vec<TimesheetRow>, <Self as TimesheetRepository<R>>::Error>;
}
