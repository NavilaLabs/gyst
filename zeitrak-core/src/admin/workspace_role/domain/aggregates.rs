use eventually::aggregate::Aggregate;
use serde::{Deserialize, Serialize};

use crate::{
    admin::{workspace::WorkspaceId, workspace_role::WorkspaceRoleEvent},
    shared::AggregateId,
};

pub type WorkspaceRoleId = AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceRole {
    id: WorkspaceRoleId,
    workspace_id: WorkspaceId,
    name: Option<String>,
}

impl WorkspaceRole {
    #[must_use]
    pub const fn id(&self) -> &WorkspaceRoleId {
        &self.id
    }

    #[must_use]
    pub const fn workspace_id(&self) -> &WorkspaceId {
        &self.workspace_id
    }

    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("workspace role already exists")]
    AlreadyExists,
    #[error("workspace role not found")]
    NotFound,
}

impl Aggregate for WorkspaceRole {
    type Id = WorkspaceRoleId;
    type Event = WorkspaceRoleEvent;
    type Error = Error;

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
                WorkspaceRoleEvent::Created {
                    id,
                    workspace_id,
                    name,
                },
            ) => Ok(Self {
                id,
                workspace_id,
                name,
            }),
            (Some(_), WorkspaceRoleEvent::Created { .. }) => Err(Error::AlreadyExists),
            (None, _) => Err(Error::NotFound),
            (
                Some(role),
                WorkspaceRoleEvent::PermissionGranted { .. }
                | WorkspaceRoleEvent::PermissionRevoked { .. },
            ) => Ok(role),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_ids() -> (WorkspaceRoleId, WorkspaceId) {
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
        let event = WorkspaceRoleEvent::Created {
            id: id.clone(),
            workspace_id: workspace_id.clone(),
            name: Some("admin".to_string()),
        };
        let result = WorkspaceRole::apply(None, event);
        assert!(result.is_ok());
        let role = result.unwrap();
        assert_eq!(role.id(), &id);
        assert_eq!(role.workspace_id(), &workspace_id);
        assert_eq!(role.name(), Some("admin"));
    }

    #[test]
    fn apply_created_event_to_existing_role_returns_already_exists() {
        let (id, workspace_id) = test_ids();
        let existing = WorkspaceRole::apply(
            None,
            WorkspaceRoleEvent::Created {
                id: id.clone(),
                workspace_id: workspace_id.clone(),
                name: None,
            },
        )
        .unwrap();
        let result = WorkspaceRole::apply(
            Some(existing),
            WorkspaceRoleEvent::Created {
                id,
                workspace_id,
                name: None,
            },
        );
        assert!(matches!(result, Err(Error::AlreadyExists)));
    }
}
