use serde::{Deserialize, Serialize};

use crate::admin::workspace;

pub type Id = crate::AggregateId;

/// A workspace represents an isolated context in which activities,
/// timesheets, and so on are tracked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregate {
    id: Id,
    name: String,
    #[serde(default)]
    is_deleted: bool,
}

impl Aggregate {
    /// The id of the workspace.
    #[must_use]
    pub const fn id(&self) -> &Id {
        &self.id
    }

    /// The name of the workspace.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the workspace is deleted or not.
    #[must_use]
    pub const fn is_deleted(&self) -> bool {
        self.is_deleted
    }
}

impl eventually::aggregate::Aggregate for Aggregate {
    type Id = workspace::Id;
    type Event = workspace::Event;
    type Error = workspace::Error;

    fn type_name() -> &'static str {
        "workspace"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match (state, event) {
            (
                None,
                workspace::Event::Created {
                    id,
                    name,
                    is_deleted,
                },
            ) => Ok(Self {
                id,
                name,
                is_deleted,
            }),
            (Some(_), workspace::Event::Created { .. }) => Err(workspace::Error::AlreadyExists),
            (None, _) => Err(workspace::Error::NotFound),
            (
                Some(w),
                workspace::Event::UserRoleAssigned { .. }
                | workspace::Event::UserRoleRevoked { .. }
                | workspace::Event::UserPermissionGranted { .. }
                | workspace::Event::UserPermissionRevoked { .. }
                | workspace::Event::UserRemoved { .. },
            ) => Ok(w),
            (Some(mut w), workspace::Event::SettingsUpdated { name }) => {
                w.name = name;
                Ok(w)
            }
        }
    }
}

impl crate::snapshot_policy::SnapshotPolicy for Aggregate {}

#[cfg(test)]
mod tests {
    use eventually::aggregate::Aggregate;

    use crate::{admin::workspace, admin::workspace::Error};

    fn test_id() -> workspace::Id {
        "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
            .parse()
            .expect("valid UUID")
    }

    #[test]
    fn apply_created_event_to_no_state_builds_workspace() {
        let id = test_id();
        let event = workspace::Event::Created {
            id: id.clone(),
            name: "Acme".to_string(),
            is_deleted: false,
        };
        let result = workspace::Aggregate::apply(None, event);
        assert!(result.is_ok());
        let workspace = result.unwrap();
        assert_eq!(workspace.id(), &id);
        assert_eq!(workspace.name(), "Acme");
    }

    #[test]
    fn apply_created_event_to_existing_workspace_returns_already_exists() {
        let id = test_id();
        let existing = workspace::Aggregate::apply(
            None,
            workspace::Event::Created {
                id: id.clone(),
                name: String::new(),
                is_deleted: false,
            },
        )
        .unwrap();
        let result = workspace::Aggregate::apply(
            Some(existing),
            workspace::Event::Created {
                id,
                name: String::new(),
                is_deleted: false,
            },
        );
        assert!(matches!(result, Err(Error::AlreadyExists)));
    }

    #[test]
    fn apply_membership_event_to_no_state_returns_not_found() {
        let user_id = "019d0ce8-facb-7c90-b9d7-287ae4f17c92"
            .parse()
            .expect("valid UUID");
        let role_id = "019d0ce8-facb-7c90-b9d7-287ae4f17c93"
            .parse()
            .expect("valid UUID");
        let result = workspace::Aggregate::apply(
            None,
            workspace::Event::UserRoleAssigned {
                user_id,
                workspace_role_id: role_id,
            },
        );
        assert!(matches!(result, Err(Error::NotFound)));
    }
}
