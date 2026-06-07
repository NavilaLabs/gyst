use eventually::aggregate::Aggregate;
use serde::{Deserialize, Serialize};

use crate::shared::AggregateId;
use crate::tenant::activity::ActivityId;
use crate::tenant::timesheet::TimesheetEvent;
use crate::tenant::timesheet::domain::events::UserId;

pub type TimesheetId = AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timesheet {
    id: TimesheetId,
    user_id: UserId,
    activity_id: Option<ActivityId>,
    start_time: String,
    end_time: Option<String>,
    duration: Option<i32>,
    description: Option<String>,
    timezone: String,
    cancelled: bool,
}

impl Timesheet {
    #[must_use]
    pub const fn id(&self) -> &TimesheetId {
        &self.id
    }

    #[must_use]
    pub const fn user_id(&self) -> &UserId {
        &self.user_id
    }

    #[must_use]
    pub const fn activity_id(&self) -> Option<&ActivityId> {
        self.activity_id.as_ref()
    }

    #[must_use]
    pub fn start_time(&self) -> &str {
        &self.start_time
    }

    #[must_use]
    pub fn end_time(&self) -> Option<&str> {
        self.end_time.as_deref()
    }

    #[must_use]
    pub const fn duration(&self) -> Option<i32> {
        self.duration
    }

    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    #[must_use]
    pub fn timezone(&self) -> &str {
        &self.timezone
    }

    #[must_use]
    pub const fn cancelled(&self) -> bool {
        self.cancelled
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("timesheet already exists")]
    AlreadyExists,
    #[error("timesheet not found")]
    NotFound,
    #[error("timesheet already exported")]
    AlreadyExported,
    #[error("timesheet already cancelled")]
    AlreadyCancelled,
}

impl Aggregate for Timesheet {
    type Id = TimesheetId;
    type Event = TimesheetEvent;
    type Error = Error;

    fn type_name() -> &'static str {
        "timesheet"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match (state, event) {
            (
                None,
                TimesheetEvent::Started {
                    id,
                    user_id,
                    activity_id,
                    start_time,
                    timezone,
                },
            ) => Ok(Self {
                id,
                user_id,
                activity_id,
                start_time,
                end_time: None,
                duration: None,
                description: None,
                timezone,
                cancelled: false,
            }),
            (Some(_), TimesheetEvent::Started { .. }) => Err(Error::AlreadyExists),
            (None, _) => Err(Error::NotFound),
            (
                Some(mut t),
                TimesheetEvent::Stopped {
                    end_time, duration, ..
                },
            ) => {
                t.end_time = Some(end_time);
                t.duration = Some(duration);
                Ok(t)
            }
            (Some(mut t), TimesheetEvent::Updated { description }) => {
                t.description = description;
                Ok(t)
            }
            (Some(mut t), TimesheetEvent::Reassigned { activity_id }) => {
                t.activity_id = Some(activity_id);
                Ok(t)
            }
            (
                Some(mut t),
                TimesheetEvent::TimeUpdated {
                    start_time,
                    end_time,
                    duration,
                },
            ) => {
                t.start_time = start_time;
                t.end_time = end_time;
                t.duration = duration;
                Ok(t)
            }
            (Some(t), TimesheetEvent::Cancelled {}) => {
                if t.cancelled {
                    return Err(Error::AlreadyCancelled);
                }
                Ok(Self {
                    cancelled: true,
                    ..t
                })
            }
        }
    }
}

impl crate::snapshot_policy::SnapshotPolicy for Timesheet {}

#[cfg(test)]
mod tests {
    use super::*;

    const TS_ID: &str = "019d0ce8-facb-7c90-b9d7-287ae4f17c91";
    const USER_ID: &str = "019d0ce8-facb-7c90-b9d7-287ae4f17c92";
    const ACT_ID: &str = "019d0ce8-facb-7c90-b9d7-287ae4f17c93";

    fn ts_id() -> TimesheetId {
        TS_ID.parse().unwrap()
    }
    fn user_id() -> UserId {
        USER_ID.parse().unwrap()
    }
    fn act_id() -> ActivityId {
        ACT_ID.parse().unwrap()
    }

    fn started_event() -> TimesheetEvent {
        TimesheetEvent::Started {
            id: ts_id(),
            user_id: user_id(),
            activity_id: Some(act_id()),
            start_time: "2024-01-01T09:00:00Z".to_string(),
            timezone: "Europe/Berlin".to_string(),
        }
    }

    fn started() -> Timesheet {
        Timesheet::apply(None, started_event()).unwrap()
    }

    #[test]
    fn apply_started_to_no_state_builds_timesheet() {
        let t = started();
        assert_eq!(t.id(), &ts_id());
        assert_eq!(t.user_id(), &user_id());
        assert_eq!(t.activity_id(), Some(&act_id()));
        assert_eq!(t.start_time(), "2024-01-01T09:00:00Z");
        assert!(t.end_time().is_none());
        assert!(t.duration().is_none());
        assert!(t.description().is_none());
        assert_eq!(t.timezone(), "Europe/Berlin");
    }

    #[test]
    fn apply_started_without_activity_id_is_valid() {
        let t = Timesheet::apply(
            None,
            TimesheetEvent::Started {
                id: ts_id(),
                user_id: user_id(),
                activity_id: None,
                start_time: "2024-01-01T09:00:00Z".to_string(),
                timezone: "UTC".to_string(),
            },
        )
        .unwrap();
        assert!(t.activity_id().is_none());
    }

    #[test]
    fn apply_started_to_existing_returns_already_exists() {
        let existing = started();
        let result = Timesheet::apply(Some(existing), started_event());
        assert!(matches!(result, Err(Error::AlreadyExists)));
    }

    #[test]
    fn apply_non_started_event_to_no_state_returns_not_found() {
        let result = Timesheet::apply(
            None,
            TimesheetEvent::Stopped {
                end_time: "2024-01-01T10:00:00Z".to_string(),
                duration: 3600,
            },
        );
        assert!(matches!(result, Err(Error::NotFound)));
    }

    #[test]
    fn apply_stopped_sets_end_time_and_duration() {
        let t = Timesheet::apply(
            Some(started()),
            TimesheetEvent::Stopped {
                end_time: "2024-01-01T10:00:00Z".to_string(),
                duration: 3600,
            },
        )
        .unwrap();
        assert_eq!(t.end_time(), Some("2024-01-01T10:00:00Z"));
        assert_eq!(t.duration(), Some(3600));
    }

    #[test]
    fn apply_updated_sets_description() {
        let t = Timesheet::apply(
            Some(started()),
            TimesheetEvent::Updated {
                description: Some("pair session".to_string()),
            },
        )
        .unwrap();
        assert_eq!(t.description(), Some("pair session"));
    }

    #[test]
    fn apply_updated_can_clear_description() {
        let with_desc = Timesheet::apply(
            Some(started()),
            TimesheetEvent::Updated {
                description: Some("note".to_string()),
            },
        )
        .unwrap();
        let cleared = Timesheet::apply(
            Some(with_desc),
            TimesheetEvent::Updated { description: None },
        )
        .unwrap();
        assert!(cleared.description().is_none());
    }

    #[test]
    fn apply_reassigned_sets_activity_id() {
        let new_act: ActivityId = "019d0ce8-facb-7c90-b9d7-287ae4f17c94".parse().unwrap();
        let t = Timesheet::apply(
            Some(started()),
            TimesheetEvent::Reassigned {
                activity_id: new_act.clone(),
            },
        )
        .unwrap();
        assert_eq!(t.activity_id(), Some(&new_act));
    }

    #[test]
    fn apply_time_updated_overwrites_all_time_fields() {
        let t = Timesheet::apply(
            Some(started()),
            TimesheetEvent::TimeUpdated {
                start_time: "2024-01-01T08:00:00Z".to_string(),
                end_time: Some("2024-01-01T09:30:00Z".to_string()),
                duration: Some(5400),
            },
        )
        .unwrap();
        assert_eq!(t.start_time(), "2024-01-01T08:00:00Z");
        assert_eq!(t.end_time(), Some("2024-01-01T09:30:00Z"));
        assert_eq!(t.duration(), Some(5400));
    }

    #[test]
    fn apply_time_updated_preserves_running_state_when_no_end_time() {
        let t = Timesheet::apply(
            Some(started()),
            TimesheetEvent::TimeUpdated {
                start_time: "2024-01-01T08:30:00Z".to_string(),
                end_time: None,
                duration: None,
            },
        )
        .unwrap();
        assert!(t.end_time().is_none());
        assert!(t.duration().is_none());
    }
}
