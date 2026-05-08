use std::fmt::Debug;

use crate::{
    shared::repositories::{ReadRepository, Repository, WriteRepository},
    tenant::timesheet_tag::domain::aggregates::TimesheetTag,
};

pub trait TimesheetTagRepository<R>: Repository<TimesheetTag, R> + Send + Sync {
    type Error: Debug
        + Send
        + Sync
        + From<<Self as ReadRepository<TimesheetTag, R>>::Error>
        + From<<Self as WriteRepository<TimesheetTag>>::Error>;
}
