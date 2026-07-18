use eventually::message::Message;
use serde::{Deserialize, Serialize};

use crate::{
    admin::user,
    workspace::{activity, timesheet},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Started {
        id: timesheet::Id,
        user_id: user::Id,
        /// None when started via quick timer — assigned later via `Reassigned`.
        activity_id: Option<activity::Id>,
        /// RFC-3339 timestamp string.
        start_time: String,
        timezone: String,
    },
    Stopped {
        /// RFC-3339 timestamp string.
        end_time: String,
        /// Duration in seconds.
        duration: i32,
    },
    Updated {
        description: Option<String>,
    },
    Reassigned {
        activity_id: activity::Id,
    },
    /// Corrects the start and/or end time of a timesheet after the fact.
    /// For a running timer `end_time` and `duration` remain `None`.
    TimeUpdated {
        /// RFC-3339 timestamp string.
        start_time: String,
        /// RFC-3339 timestamp string. `None` if the timer is still running.
        end_time: Option<String>,
        /// Duration in seconds. `None` if the timer is still running.
        duration: Option<i32>,
    },
    /// Soft-cancels the timesheet — it is excluded from queries and reporting.
    Cancelled {},
}

impl Message for Event {
    fn name(&self) -> &'static str {
        match self {
            Self::Started { .. } => "TimesheetStarted",
            Self::Stopped { .. } => "TimesheetStopped",
            Self::Updated { .. } => "TimesheetUpdated",
            Self::Reassigned { .. } => "TimesheetReassigned",
            Self::TimeUpdated { .. } => "TimesheetTimeUpdated",
            Self::Cancelled { .. } => "TimesheetCancelled",
        }
    }
}
