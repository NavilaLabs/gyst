use serde::{Deserialize, Serialize};

use crate::{AggregateId, workspace::timesheet_tag};

pub type Id = AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregate {
    id: Id,
    name: String,
}

impl Aggregate {
    #[must_use]
    pub const fn id(&self) -> &Id {
        &self.id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl eventually::aggregate::Aggregate for Aggregate {
    type Id = Id;
    type Event = timesheet_tag::Event;
    type Error = timesheet_tag::Error;

    fn type_name() -> &'static str {
        "timesheet_tag"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match (state, event) {
            (None, timesheet_tag::Event::Created { id, name }) => Ok(Self { id, name }),
            (Some(_), timesheet_tag::Event::Created { .. }) => {
                Err(timesheet_tag::Error::AlreadyExists)
            }
            (None, _) => Err(timesheet_tag::Error::NotFound),
            (Some(mut t), timesheet_tag::Event::Renamed { name }) => {
                t.name = name;
                Ok(t)
            }
            (
                Some(t),
                timesheet_tag::Event::TimesheetTagged { .. }
                | timesheet_tag::Event::TimesheetUntagged { .. }
                | timesheet_tag::Event::Deleted { .. },
            ) => Ok(t),
        }
    }
}

impl crate::snapshot_policy::SnapshotPolicy for Aggregate {}

#[cfg(test)]
mod tests {
    use eventually::aggregate::Aggregate;

    use crate::workspace::{timesheet, timesheet_tag};

    fn test_id() -> timesheet_tag::Id {
        "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
            .parse()
            .expect("valid UUID")
    }

    fn created(id: timesheet_tag::Id, name: &str) -> timesheet_tag::Event {
        timesheet_tag::Event::Created {
            id,
            name: name.to_string(),
        }
    }

    #[test]
    fn apply_created_to_no_state_builds_tag() {
        let id = test_id();
        let t = timesheet_tag::Aggregate::apply(None, created(id.clone(), "backend")).unwrap();
        assert_eq!(t.id(), &id);
        assert_eq!(t.name(), "backend");
    }

    #[test]
    fn apply_created_to_existing_tag_returns_already_exists() {
        let id = test_id();
        let existing =
            timesheet_tag::Aggregate::apply(None, created(id.clone(), "backend")).unwrap();
        let result = timesheet_tag::Aggregate::apply(Some(existing), created(id, "other"));
        assert!(matches!(result, Err(timesheet_tag::Error::AlreadyExists)));
    }

    #[test]
    fn apply_non_created_to_no_state_returns_not_found() {
        let result = timesheet_tag::Aggregate::apply(
            None,
            timesheet_tag::Event::Renamed {
                name: "x".to_string(),
            },
        );
        assert!(matches!(result, Err(timesheet_tag::Error::NotFound)));
    }

    #[test]
    fn apply_renamed_mutates_name() {
        let id = test_id();
        let existing = timesheet_tag::Aggregate::apply(None, created(id, "old-name")).unwrap();
        let t = timesheet_tag::Aggregate::apply(
            Some(existing),
            timesheet_tag::Event::Renamed {
                name: "new-name".to_string(),
            },
        )
        .unwrap();
        assert_eq!(t.name(), "new-name");
    }

    #[test]
    fn apply_timesheet_tagged_preserves_tag_state() {
        let id = test_id();
        let existing =
            timesheet_tag::Aggregate::apply(None, created(id.clone(), "backend")).unwrap();
        let ts_id: timesheet::Id = "019d0ce8-facb-7c90-b9d7-287ae4f17c92".parse().unwrap();
        let t = timesheet_tag::Aggregate::apply(
            Some(existing),
            timesheet_tag::Event::TimesheetTagged {
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
        let existing =
            timesheet_tag::Aggregate::apply(None, created(id.clone(), "backend")).unwrap();
        let ts_id: timesheet::Id = "019d0ce8-facb-7c90-b9d7-287ae4f17c92".parse().unwrap();
        let t = timesheet_tag::Aggregate::apply(
            Some(existing),
            timesheet_tag::Event::TimesheetUntagged {
                timesheet_id: ts_id,
            },
        )
        .unwrap();
        assert_eq!(t.id(), &id);
    }
}
