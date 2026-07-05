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

    /// Returns a page of non-cancelled timesheets for a user, newest first.
    ///
    /// Returns `(rows, total_count)` where `total_count` is the full un-paged count.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn recent_for_user(
        &self,
        user_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<TimesheetRow>, u64), <Self as TimesheetRepository<R>>::Error>;

    /// Returns the running timesheet for a user (`end_time` IS NULL), if any.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn running_for_user(
        &self,
        user_id: &str,
    ) -> Result<Option<TimesheetRow>, <Self as TimesheetRepository<R>>::Error>;

    /// Returns a page of non-cancelled timesheets across all users in the workspace,
    /// newest first. `member_id` optionally restricts results to a single user.
    ///
    /// Returns `(rows, total_count)` where `total_count` is the full un-paged count.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn recent_for_workspace(
        &self,
        page: u32,
        page_size: u32,
        member_id: Option<&str>,
    ) -> Result<(Vec<TimesheetRow>, u64), <Self as TimesheetRepository<R>>::Error>;

    /// Returns all completed (end\_time IS NOT NULL, not cancelled) timesheets
    /// with `start_time >= since_rfc3339`, for a single member or all members.
    ///
    /// Used for dashboard KPI and chart aggregation.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn stats_for_period(
        &self,
        member_id: Option<&str>,
        since_rfc3339: &str,
    ) -> Result<Vec<TimesheetRow>, <Self as TimesheetRepository<R>>::Error>;

    /// Returns a page of non-cancelled timesheets (including running), newest first.
    ///
    /// `from` and `to` are optional RFC-3339 bounds on `start_time`.
    /// `member_id` optionally restricts results to a single user.
    /// Returns `(rows, total_count)`.
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
    ) -> Result<(Vec<TimesheetRow>, u64), <Self as TimesheetRepository<R>>::Error>;

    /// Returns all completed timesheets in the optional date range, for metrics computation.
    ///
    /// `from` and `to` are optional RFC-3339 bounds on `start_time`.
    /// `member_id` optionally restricts results to a single user.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn stats_for_timeline(
        &self,
        from: Option<&str>,
        to: Option<&str>,
        member_id: Option<&str>,
    ) -> Result<Vec<TimesheetRow>, <Self as TimesheetRepository<R>>::Error>;
}
