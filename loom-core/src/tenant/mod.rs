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

impl<Repo, Row, Agg> From<activity::DomainError> for crate::Error<Repo, Row, Agg>
where
    Row: Debug,
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Row> + WriteRepository<Agg>,
{
    fn from(value: activity::DomainError) -> Self {
        Self::TenantDatabaseError(Error::ActivityError(value.into()))
    }
}

impl<Repo, Row, Agg> From<timesheet::DomainError> for crate::Error<Repo, Row, Agg>
where
    Row: Debug,
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Row> + WriteRepository<Agg>,
{
    fn from(value: timesheet::DomainError) -> Self {
        Self::TenantDatabaseError(Error::TimesheetError(value.into()))
    }
}

impl<Repo, Row, Agg> From<timesheet_tag::DomainError> for crate::Error<Repo, Row, Agg>
where
    Row: Debug,
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Row> + WriteRepository<Agg>,
{
    fn from(value: timesheet_tag::DomainError) -> Self {
        Self::TenantDatabaseError(Error::TagError(value.into()))
    }
}
