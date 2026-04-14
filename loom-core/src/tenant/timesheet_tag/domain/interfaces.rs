use async_trait::async_trait;

use crate::tenant::timesheet_tag::domain::aggregates::TimesheetTag;
use eventually::aggregate::repository::{Getter, Saver};

#[async_trait]
pub trait TimesheetTagRepository: Getter<TimesheetTag> + Saver<TimesheetTag> + Send + Sync {}
