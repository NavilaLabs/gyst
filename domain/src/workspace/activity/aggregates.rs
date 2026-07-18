use serde::{Deserialize, Serialize};

use crate::{AggregateId, workspace::activity};

pub type Id = AggregateId;

/// An activity to track time for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregate {
    id: Id,
    name: String,
    color: String,
    comment: Option<String>,
}

impl Aggregate {
    /// The id of the activity.
    #[must_use]
    pub const fn id(&self) -> &Id {
        &self.id
    }

    /// The name of the activity.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The color of the activity to better distinguish visually.
    #[must_use]
    pub fn color(&self) -> &str {
        &self.color
    }

    /// An optional comment for the activity.
    #[must_use]
    pub const fn comment(&self) -> Option<&String> {
        self.comment.as_ref()
    }
}

impl eventually::aggregate::Aggregate for Aggregate {
    type Id = Id;
    type Event = activity::Event;
    type Error = activity::Error;

    fn type_name() -> &'static str {
        "activity"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match (state, event) {
            (
                None,
                activity::Event::Created {
                    id,
                    name,
                    color,
                    comment,
                },
            ) => Ok(Self {
                id,
                name,
                color,
                comment,
            }),
            (Some(_), activity::Event::Created { .. }) => Err(activity::Error::AlreadyExists),
            (None, _) => Err(activity::Error::NotFound),
            (
                Some(mut a),
                activity::Event::Updated {
                    name,
                    color,
                    comment,
                },
            ) => {
                a.name = name;
                a.color = color;
                a.comment = comment;
                Ok(a)
            }
            (Some(a), activity::Event::Deleted {}) => Ok(a),
        }
    }
}

impl crate::snapshot_policy::SnapshotPolicy for Aggregate {}

#[cfg(test)]
mod tests {
    use eventually::aggregate::Aggregate;

    use crate::workspace::activity;

    fn test_id() -> activity::Id {
        "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
            .parse()
            .expect("valid UUID")
    }

    fn created(
        id: activity::Id,
        name: &str,
        color: &str,
        comment: Option<&str>,
    ) -> activity::Event {
        activity::Event::Created {
            id,
            name: name.to_string(),
            color: color.to_string(),
            comment: comment.map(str::to_owned),
        }
    }

    #[test]
    fn apply_created_to_no_state_builds_activity() {
        let id = test_id();
        let a = activity::Aggregate::apply(None, created(id.clone(), "Debug", "#3b82f6", None))
            .unwrap();
        assert_eq!(a.id(), &id);
        assert_eq!(a.name(), "Debug");
        assert_eq!(a.color(), "#3b82f6");
        assert!(a.comment().is_none());
    }

    #[test]
    fn apply_created_stores_comment_when_provided() {
        let id = test_id();
        let a = activity::Aggregate::apply(None, created(id, "Test", "#22c55e", Some("a note")))
            .unwrap();
        assert_eq!(a.comment(), Some(&"a note".to_string()));
    }

    #[test]
    fn apply_created_to_existing_activity_returns_already_exists() {
        let id = test_id();
        let existing =
            activity::Aggregate::apply(None, created(id.clone(), "First", "#22c55e", None))
                .unwrap();
        let result = Aggregate::apply(Some(existing), created(id, "Second", "#3b82f6", None));
        assert!(matches!(result, Err(activity::Error::AlreadyExists)));
    }

    #[test]
    fn apply_updated_to_no_state_returns_not_found() {
        let result = activity::Aggregate::apply(
            None,
            activity::Event::Updated {
                name: "X".to_string(),
                color: "#3b82f6".to_string(),
                comment: None,
            },
        );
        assert!(matches!(result, Err(activity::Error::NotFound)));
    }

    #[test]
    fn apply_updated_mutates_name_color_and_comment() {
        let id = test_id();
        let existing =
            activity::Aggregate::apply(None, created(id, "Old", "#22c55e", None)).unwrap();
        let updated = activity::Aggregate::apply(
            Some(existing),
            activity::Event::Updated {
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
            activity::Aggregate::apply(None, created(id, "Old", "#22c55e", Some("original note")))
                .unwrap();
        let updated = Aggregate::apply(
            Some(existing),
            activity::Event::Updated {
                name: "New".to_string(),
                color: "#22c55e".to_string(),
                comment: None,
            },
        )
        .unwrap();
        assert!(updated.comment().is_none());
    }
}
