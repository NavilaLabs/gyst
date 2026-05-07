pub mod activity;
pub mod timesheet;
pub mod timesheet_tag;

use std::fmt::Debug;

use eventually::aggregate::Aggregate;

use crate::shared::repositories::{ReadRepository, WriteRepository};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0:?}")]
    ActivityError(#[from] activity::Error),
    #[error("{0:?}")]
    TimesheetError(#[from] timesheet::Error),
    #[error("{0:?}")]
    TagError(#[from] timesheet_tag::Error),
}

impl<Repo, Agg> From<activity::Error> for crate::Error<Repo, Agg>
where
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Agg> + WriteRepository<Agg>,
{
    fn from(value: activity::Error) -> Self {
        Self::TenantError(Error::ActivityError(value))
    }
}

impl<Repo, Agg> From<timesheet::Error> for crate::Error<Repo, Agg>
where
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Agg> + WriteRepository<Agg>,
{
    fn from(value: timesheet::Error) -> Self {
        Self::TenantError(Error::TimesheetError(value))
    }
}

impl<Repo, Agg> From<timesheet_tag::Error> for crate::Error<Repo, Agg>
where
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Agg> + WriteRepository<Agg>,
{
    fn from(value: timesheet_tag::Error) -> Self {
        Self::TenantError(Error::TagError(value))
    }
}
