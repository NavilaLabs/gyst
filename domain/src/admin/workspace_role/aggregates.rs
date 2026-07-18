use serde::{Deserialize, Serialize};

use crate::admin::{workspace, workspace_role};

pub type Id = crate::AggregateId;

#[allow(clippy::doc_markdown)]
/// A role to assign to a workspace's member.
///
/// Default workspace roles (via migration):
/// 1. standard: The role a workspace member gets assigned to by default.
/// 2. workspace_admin: The role for managing the workspaces. This includes
///    managing roles and their permissions, members, and so on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregate {
    id: Id,
    workspace_id: workspace::Id,
    name: String,
    #[serde(default)]
    is_deleted: bool,
}

impl Aggregate {
    /// The id of the workspace role.
    #[must_use]
    pub const fn id(&self) -> &Id {
        &self.id
    }

    /// The if of the workspace the role belongs to.
    #[must_use]
    pub const fn workspace_id(&self) -> &workspace::Id {
        &self.workspace_id
    }

    /// The name of the workspace role.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the workspace role is deleted.
    #[must_use]
    pub const fn is_deleted(&self) -> bool {
        self.is_deleted
    }
}

impl eventually::aggregate::Aggregate for workspace_role::Aggregate {
    type Id = workspace_role::Id;
    type Event = workspace_role::Event;
    type Error = workspace_role::Error;

    fn type_name() -> &'static str {
        "workspace_role"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match (state, event) {
            (
                None,
                workspace_role::Event::Created {
                    id,
                    workspace_id,
                    name,
                },
            ) => Ok(Self {
                id,
                workspace_id,
                name,
                is_deleted: false,
            }),
            (Some(_), workspace_role::Event::Created { .. }) => {
                Err(workspace_role::Error::AlreadyExists)
            }
            (None, _) => Err(workspace_role::Error::NotFound),
            (
                Some(role),
                workspace_role::Event::PermissionGranted { .. }
                | workspace_role::Event::PermissionRevoked { .. },
            ) => Ok(role),
            (Some(role), workspace_role::Event::Renamed { name }) => Ok(Self { name, ..role }),
            (Some(role), workspace_role::Event::Deleted) => Ok(Self {
                is_deleted: true,
                ..role
            }),
        }
    }
}

impl crate::snapshot_policy::SnapshotPolicy for workspace_role::Aggregate {}

#[cfg(test)]
mod tests {
    use eventually::aggregate::Aggregate;

    use crate::admin::workspace_role::{self, Error};

    fn test_ids() -> (workspace_role::Id, workspace_role::Id) {
        (
            "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
                .parse()
                .expect("valid UUID"),
            "019d0ce8-facb-7c90-b9d7-287ae4f17c92"
                .parse()
                .expect("valid UUID"),
        )
    }

    #[test]
    fn apply_created_event_to_no_state_builds_role() {
        let (id, workspace_id) = test_ids();
        let event = workspace_role::Event::Created {
            id: id.clone(),
            workspace_id: workspace_id.clone(),
            name: "admin".to_string(),
        };
        let result = workspace_role::Aggregate::apply(None, event);
        assert!(result.is_ok());
        let role = result.unwrap();
        assert_eq!(role.id(), &id);
        assert_eq!(role.workspace_id(), &workspace_id);
        assert_eq!(role.name(), "admin");
    }

    #[test]
    fn apply_created_event_to_existing_role_returns_already_exists() {
        let (id, workspace_id) = test_ids();
        let existing = workspace_role::Aggregate::apply(
            None,
            workspace_role::Event::Created {
                id: id.clone(),
                workspace_id: workspace_id.clone(),
                name: String::new(),
            },
        )
        .unwrap();
        let result = workspace_role::Aggregate::apply(
            Some(existing),
            workspace_role::Event::Created {
                id,
                workspace_id,
                name: String::new(),
            },
        );
        assert!(matches!(result, Err(Error::AlreadyExists)));
    }
}
