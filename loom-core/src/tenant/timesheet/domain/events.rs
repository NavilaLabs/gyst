use eventually::message::Message;
use serde::{Deserialize, Serialize};

use crate::shared::AggregateId;
use crate::tenant::activity::ActivityId;
use crate::tenant::timesheet::TimesheetId;

/// User ID references an admin-domain user — stored as a plain `AggregateId`.
pub type UserId = AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimesheetEvent {
    Started {
        id: TimesheetId,
        user_id: UserId,
        /// None when started via quick timer — assigned later via `Reassigned`.
        activity_id: Option<ActivityId>,
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
        activity_id: ActivityId,
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
}

impl Message for TimesheetEvent {
    fn name(&self) -> &'static str {
        match self {
            Self::Started { .. } => "TimesheetStarted",
            Self::Stopped { .. } => "TimesheetStopped",
            Self::Updated { .. } => "TimesheetUpdated",
            Self::Reassigned { .. } => "TimesheetReassigned",
            Self::TimeUpdated { .. } => "TimesheetTimeUpdated",
        }
    }
}
