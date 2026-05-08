use std::fmt::Debug;

use crate::{
    shared::repositories::{ReadRepository, Repository, WriteRepository},
    tenant::timesheet::domain::aggregates::Timesheet,
};

pub trait TimesheetRepository<R>: Repository<Timesheet, R> + Send + Sync {
    type Error: Debug
        + Send
        + Sync
        + From<<Self as ReadRepository<Timesheet, R>>::Error>
        + From<<Self as WriteRepository<Timesheet>>::Error>;
}
