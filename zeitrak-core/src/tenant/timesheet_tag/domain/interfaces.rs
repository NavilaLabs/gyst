use std::fmt::Debug;

use crate::{
    shared::repositories::{ReadRepository, WriteRepository},
    tenant::timesheet_tag::domain::aggregates::TimesheetTag,
};

pub trait TimesheetTagRepository:
    ReadRepository<TimesheetTag> + WriteRepository<TimesheetTag> + Send + Sync
{
    type Error: Debug
        + Send
        + Sync
        + From<<Self as ReadRepository<TimesheetTag>>::Error>
        + From<<Self as WriteRepository<TimesheetTag>>::Error>;
}
