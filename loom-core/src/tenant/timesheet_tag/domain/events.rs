use eventually::message::Message;
use serde::{Deserialize, Serialize};

use crate::tenant::timesheet::TimesheetId;
use crate::tenant::timesheet_tag::TimesheetTagId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimesheetTagEvent {
    Created { id: TimesheetTagId, name: String },
    Renamed { name: String },
    TimesheetTagged { timesheet_id: TimesheetId },
    TimesheetUntagged { timesheet_id: TimesheetId },
}

impl Message for TimesheetTagEvent {
    fn name(&self) -> &'static str {
        match self {
            Self::Created { .. } => "TagCreated",
            Self::Renamed { .. } => "TagRenamed",
            Self::TimesheetTagged { .. } => "TagTimesheetTagged",
            Self::TimesheetUntagged { .. } => "TagTimesheetUntagged",
        }
    }
}
