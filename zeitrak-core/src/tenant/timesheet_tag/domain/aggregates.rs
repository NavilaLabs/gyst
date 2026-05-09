use eventually::aggregate::Aggregate;
use serde::{Deserialize, Serialize};

use crate::shared::AggregateId;
use crate::tenant::timesheet_tag::TimesheetTagEvent;

pub type TimesheetTagId = AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimesheetTag {
    id: TimesheetTagId,
    name: String,
}

impl TimesheetTag {
    #[must_use]
    pub const fn id(&self) -> &TimesheetTagId {
        &self.id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

crate::aggregate_errors!("tag");

impl Aggregate for TimesheetTag {
    type Id = TimesheetTagId;
    type Event = TimesheetTagEvent;
    type Error = Error;

    fn type_name() -> &'static str {
        "timesheet_tag"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match (state, event) {
            (None, TimesheetTagEvent::Created { id, name }) => Ok(Self { id, name }),
            (Some(_), TimesheetTagEvent::Created { .. }) => Err(Error::AlreadyExists),
            (None, _) => Err(Error::NotFound),
            (Some(mut t), TimesheetTagEvent::Renamed { name }) => {
                t.name = name;
                Ok(t)
            }
            (
                Some(t),
                TimesheetTagEvent::TimesheetTagged { .. }
                | TimesheetTagEvent::TimesheetUntagged { .. }
                | TimesheetTagEvent::Deleted { .. },
            ) => Ok(t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_id() -> TimesheetTagId {
        "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
            .parse()
            .expect("valid UUID")
    }

    fn created(id: TimesheetTagId, name: &str) -> TimesheetTagEvent {
        TimesheetTagEvent::Created {
            id,
            name: name.to_string(),
        }
    }

    #[test]
    fn apply_created_to_no_state_builds_tag() {
        let id = test_id();
        let t = TimesheetTag::apply(None, created(id.clone(), "backend")).unwrap();
        assert_eq!(t.id(), &id);
        assert_eq!(t.name(), "backend");
    }

    #[test]
    fn apply_created_to_existing_tag_returns_already_exists() {
        let id = test_id();
        let existing = TimesheetTag::apply(None, created(id.clone(), "backend")).unwrap();
        let result = TimesheetTag::apply(Some(existing), created(id, "other"));
        assert!(matches!(result, Err(Error::AlreadyExists)));
    }

    #[test]
    fn apply_non_created_to_no_state_returns_not_found() {
        let result = TimesheetTag::apply(
            None,
            TimesheetTagEvent::Renamed {
                name: "x".to_string(),
            },
        );
        assert!(matches!(result, Err(Error::NotFound)));
    }

    #[test]
    fn apply_renamed_mutates_name() {
        let id = test_id();
        let existing = TimesheetTag::apply(None, created(id, "old-name")).unwrap();
        let t = TimesheetTag::apply(
            Some(existing),
            TimesheetTagEvent::Renamed {
                name: "new-name".to_string(),
            },
        )
        .unwrap();
        assert_eq!(t.name(), "new-name");
    }

    #[test]
    fn apply_timesheet_tagged_preserves_tag_state() {
        let id = test_id();
        let existing = TimesheetTag::apply(None, created(id.clone(), "backend")).unwrap();
        let ts_id: crate::tenant::timesheet::TimesheetId =
            "019d0ce8-facb-7c90-b9d7-287ae4f17c92".parse().unwrap();
        let t = TimesheetTag::apply(
            Some(existing),
            TimesheetTagEvent::TimesheetTagged {
                timesheet_id: ts_id,
            },
        )
        .unwrap();
        // Tag identity is unchanged — tagging is recorded in the event stream only.
        assert_eq!(t.id(), &id);
        assert_eq!(t.name(), "backend");
    }

    #[test]
    fn apply_timesheet_untagged_preserves_tag_state() {
        let id = test_id();
        let existing = TimesheetTag::apply(None, created(id.clone(), "backend")).unwrap();
        let ts_id: crate::tenant::timesheet::TimesheetId =
            "019d0ce8-facb-7c90-b9d7-287ae4f17c92".parse().unwrap();
        let t = TimesheetTag::apply(
            Some(existing),
            TimesheetTagEvent::TimesheetUntagged {
                timesheet_id: ts_id,
            },
        )
        .unwrap();
        assert_eq!(t.id(), &id);
    }
}
