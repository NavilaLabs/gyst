use std::fmt::Debug;

use crate::{
    shared::repositories::{ReadRepository, WriteRepository},
    tenant::timesheet::domain::aggregates::Timesheet,
};

pub trait TimesheetRepository:
    ReadRepository<Timesheet> + WriteRepository<Timesheet> + Send + Sync
{
    type Error: Debug
        + Send
        + Sync
        + From<<Self as ReadRepository<Timesheet>>::Error>
        + From<<Self as WriteRepository<Timesheet>>::Error>;
}
