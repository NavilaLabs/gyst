pub mod activity;
pub mod timesheet;
pub mod timesheet_tag;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0:?}")]
    ActivityError(#[from] activity::Error),
    #[error("{0:?}")]
    TimesheetError(#[from] timesheet::Error),
    #[error("{0:?}")]
    TagError(#[from] timesheet_tag::Error),
}

impl From<activity::DomainError> for crate::Error {
    fn from(value: activity::DomainError) -> Self {
        Self::TenantDatabaseError(Error::ActivityError(value.into()))
    }
}

impl From<timesheet::DomainError> for crate::Error {
    fn from(value: timesheet::DomainError) -> Self {
        Self::TenantDatabaseError(Error::TimesheetError(value.into()))
    }
}

impl From<timesheet_tag::DomainError> for crate::Error {
    fn from(value: timesheet_tag::DomainError) -> Self {
        Self::TenantDatabaseError(Error::TagError(value.into()))
    }
}
