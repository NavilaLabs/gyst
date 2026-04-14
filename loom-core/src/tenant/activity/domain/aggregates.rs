use eventually::aggregate::Aggregate;
use serde::{Deserialize, Serialize};

use crate::shared::AggregateId;
use crate::tenant::activity::ActivityEvent;

pub type ActivityId = AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Activity {
    id: ActivityId,
    name: String,
    comment: Option<String>,
}

impl Activity {
    #[must_use]
    pub const fn id(&self) -> &ActivityId {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn comment(&self) -> Option<&String> {
        self.comment.as_ref()
    }
}

crate::aggregate_errors!("activity");

impl Aggregate for Activity {
    type Id = ActivityId;
    type Event = ActivityEvent;
    type Error = Error;

    fn type_name() -> &'static str {
        "activity"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match (state, event) {
            (None, ActivityEvent::Created { id, name, comment }) => Ok(Self { id, name, comment }),
            (Some(_), ActivityEvent::Created { .. }) => Err(Error::AlreadyExists),
            (None, _) => Err(Error::NotFound),
            (Some(mut a), ActivityEvent::Updated { name, comment, .. }) => {
                a.name = name;
                a.comment = comment;
                Ok(a)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_id() -> ActivityId {
        "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
            .parse()
            .expect("valid UUID")
    }

    fn created(id: ActivityId, name: &str, comment: Option<&str>) -> ActivityEvent {
        ActivityEvent::Created {
            id,
            name: name.to_string(),
            comment: comment.map(str::to_owned),
        }
    }

    #[test]
    fn apply_created_to_no_state_builds_activity() {
        let id = test_id();
        let a = Activity::apply(None, created(id.clone(), "Debug", None)).unwrap();
        assert_eq!(a.id(), &id);
        assert_eq!(a.name(), "Debug");
        assert!(a.comment().is_none());
    }

    #[test]
    fn apply_created_stores_comment_when_provided() {
        let id = test_id();
        let a = Activity::apply(None, created(id, "Test", Some("a note"))).unwrap();
        assert_eq!(a.comment(), Some(&"a note".to_string()));
    }

    #[test]
    fn apply_created_to_existing_activity_returns_already_exists() {
        let id = test_id();
        let existing = Activity::apply(None, created(id.clone(), "First", None)).unwrap();
        let result = Activity::apply(Some(existing), created(id, "Second", None));
        assert!(matches!(result, Err(Error::AlreadyExists)));
    }

    #[test]
    fn apply_updated_to_no_state_returns_not_found() {
        let result = Activity::apply(
            None,
            ActivityEvent::Updated {
                name: "X".to_string(),
                comment: None,
            },
        );
        assert!(matches!(result, Err(Error::NotFound)));
    }

    #[test]
    fn apply_updated_mutates_name_and_comment() {
        let id = test_id();
        let existing = Activity::apply(None, created(id, "Old", None)).unwrap();
        let updated = Activity::apply(
            Some(existing),
            ActivityEvent::Updated {
                name: "New".to_string(),
                comment: Some("note".to_string()),
            },
        )
        .unwrap();
        assert_eq!(updated.name(), "New");
        assert_eq!(updated.comment(), Some(&"note".to_string()));
    }

    #[test]
    fn apply_updated_can_clear_comment() {
        let id = test_id();
        let existing = Activity::apply(None, created(id, "Old", Some("original note"))).unwrap();
        let updated = Activity::apply(
            Some(existing),
            ActivityEvent::Updated {
                name: "New".to_string(),
                comment: None,
            },
        )
        .unwrap();
        assert!(updated.comment().is_none());
    }
}
