use std::collections::HashMap;
use std::fmt::Debug;

use crate::{
    shared::repositories::{ReadRepository, Repository, WriteRepository},
    tenant::timesheet_tag::{
        application::views::TimesheetTagRow, domain::aggregates::TimesheetTag,
    },
};
use async_trait::async_trait;

#[async_trait]
pub trait TimesheetTagRepository<R>: Repository<TimesheetTag, R> + Send + Sync {
    type Error: Debug
        + Send
        + Sync
        + From<<Self as ReadRepository<TimesheetTag, R>>::Error>
        + From<<Self as WriteRepository<TimesheetTag>>::Error>;

    /// Returns all tags ordered by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn list_all(
        &self,
    ) -> Result<Vec<TimesheetTagRow>, <Self as TimesheetTagRepository<R>>::Error>;

    /// Returns all tags assigned to the given timesheet.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn for_timesheet(
        &self,
        timesheet_id: &str,
    ) -> Result<Vec<TimesheetTagRow>, <Self as TimesheetTagRepository<R>>::Error>;

    /// Returns all tag assignments for the given timesheet IDs as a map of `timesheet_id` → tags.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn for_timesheets_batch(
        &self,
        timesheet_ids: &[&str],
    ) -> Result<HashMap<String, Vec<TimesheetTagRow>>, <Self as TimesheetTagRepository<R>>::Error>;
}
