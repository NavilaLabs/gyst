use eventually::aggregate::Aggregate;
use serde::{Deserialize, Serialize};

use crate::shared::AggregateId;
use crate::tenant::activity::ActivityEvent;

pub type ActivityId = AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Activity {
    id: ActivityId,
    name: String,
    color: String,
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
    pub fn color(&self) -> &str {
        &self.color
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
            (None, ActivityEvent::Created { id, name, color, comment }) => {
                Ok(Self { id, name, color, comment })
            }
            (Some(_), ActivityEvent::Created { .. }) => Err(Error::AlreadyExists),
            (None, _) => Err(Error::NotFound),
            (Some(mut a), ActivityEvent::Updated { name, color, comment }) => {
                a.name = name;
                a.color = color;
                a.comment = comment;
                Ok(a)
            }
            (Some(a), ActivityEvent::Deleted {}) => Ok(a),
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

    fn created(id: ActivityId, name: &str, color: &str, comment: Option<&str>) -> ActivityEvent {
        ActivityEvent::Created {
            id,
            name: name.to_string(),
            color: color.to_string(),
            comment: comment.map(str::to_owned),
        }
    }

    #[test]
    fn apply_created_to_no_state_builds_activity() {
        let id = test_id();
        let a = Activity::apply(None, created(id.clone(), "Debug", "#3b82f6", None)).unwrap();
        assert_eq!(a.id(), &id);
        assert_eq!(a.name(), "Debug");
        assert_eq!(a.color(), "#3b82f6");
        assert!(a.comment().is_none());
    }

    #[test]
    fn apply_created_stores_comment_when_provided() {
        let id = test_id();
        let a = Activity::apply(None, created(id, "Test", "#22c55e", Some("a note"))).unwrap();
        assert_eq!(a.comment(), Some(&"a note".to_string()));
    }

    #[test]
    fn apply_created_to_existing_activity_returns_already_exists() {
        let id = test_id();
        let existing =
            Activity::apply(None, created(id.clone(), "First", "#22c55e", None)).unwrap();
        let result = Activity::apply(Some(existing), created(id, "Second", "#3b82f6", None));
        assert!(matches!(result, Err(Error::AlreadyExists)));
    }

    #[test]
    fn apply_updated_to_no_state_returns_not_found() {
        let result = Activity::apply(
            None,
            ActivityEvent::Updated {
                name: "X".to_string(),
                color: "#3b82f6".to_string(),
                comment: None,
            },
        );
        assert!(matches!(result, Err(Error::NotFound)));
    }

    #[test]
    fn apply_updated_mutates_name_color_and_comment() {
        let id = test_id();
        let existing = Activity::apply(None, created(id, "Old", "#22c55e", None)).unwrap();
        let updated = Activity::apply(
            Some(existing),
            ActivityEvent::Updated {
                name: "New".to_string(),
                color: "#a855f7".to_string(),
                comment: Some("note".to_string()),
            },
        )
        .unwrap();
        assert_eq!(updated.name(), "New");
        assert_eq!(updated.color(), "#a855f7");
        assert_eq!(updated.comment(), Some(&"note".to_string()));
    }

    #[test]
    fn apply_updated_can_clear_comment() {
        let id = test_id();
        let existing =
            Activity::apply(None, created(id, "Old", "#22c55e", Some("original note"))).unwrap();
        let updated = Activity::apply(
            Some(existing),
            ActivityEvent::Updated {
                name: "New".to_string(),
                color: "#22c55e".to_string(),
                comment: None,
            },
        )
        .unwrap();
        assert!(updated.comment().is_none());
    }
}
