use eventually::message::Message;
use serde::{Deserialize, Serialize};

use crate::workspace::{timesheet, timesheet_tag};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Created { id: timesheet_tag::Id, name: String },
    Renamed { name: String },
    TimesheetTagged { timesheet_id: timesheet::Id },
    TimesheetUntagged { timesheet_id: timesheet::Id },
    Deleted {},
}

impl Message for Event {
    fn name(&self) -> &'static str {
        match self {
            Self::Created { .. } => "TagCreated",
            Self::Renamed { .. } => "TagRenamed",
            Self::TimesheetTagged { .. } => "TagTimesheetTagged",
            Self::TimesheetUntagged { .. } => "TagTimesheetUntagged",
            Self::Deleted { .. } => "TagDeleted",
        }
    }
}
