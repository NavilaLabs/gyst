use eventually::aggregate::Aggregate;
use serde::{Deserialize, Serialize};

use crate::{admin::workspace::WorkspaceEvent, shared::AggregateId};

pub type WorkspaceId = AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    id: WorkspaceId,
    name: Option<String>,
    pub timezone: String,
    pub date_format: String,
    pub currency: String,
    pub week_start: String,
}

impl Workspace {
    #[must_use]
    pub const fn id(&self) -> &WorkspaceId {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("workspace already exists")]
    AlreadyExists,
    #[error("workspace not found")]
    NotFound,
}

impl Aggregate for Workspace {
    type Id = WorkspaceId;
    type Event = WorkspaceEvent;
    type Error = Error;

    fn type_name() -> &'static str {
        "workspace"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match (state, event) {
            (None, WorkspaceEvent::Created { id, name }) => Ok(Self {
                id,
                name,
                timezone: "Europe/Berlin".to_string(),
                date_format: "%Y-%m-%d".to_string(),
                currency: "EUR".to_string(),
                week_start: "monday".to_string(),
            }),
            (Some(_), WorkspaceEvent::Created { .. }) => Err(Error::AlreadyExists),
            (None, _) => Err(Error::NotFound),
            (
                Some(workspace),
                WorkspaceEvent::UserRoleAssigned { .. }
                | WorkspaceEvent::UserRoleRevoked { .. }
                | WorkspaceEvent::UserPermissionGranted { .. }
                | WorkspaceEvent::UserPermissionRevoked { .. },
            ) => Ok(workspace),
            (
                Some(mut workspace),
                WorkspaceEvent::SettingsUpdated {
                    name,
                    timezone,
                    date_format,
                    currency,
                    week_start,
                },
            ) => {
                workspace.name = name;
                workspace.timezone = timezone;
                workspace.date_format = date_format;
                workspace.currency = currency;
                workspace.week_start = week_start;
                Ok(workspace)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_id() -> WorkspaceId {
        "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
            .parse()
            .expect("valid UUID")
    }

    #[test]
    fn apply_created_event_to_no_state_builds_workspace() {
        let id = test_id();
        let event = WorkspaceEvent::Created {
            id: id.clone(),
            name: Some("Acme".to_string()),
        };
        let result = Workspace::apply(None, event);
        assert!(result.is_ok());
        let workspace = result.unwrap();
        assert_eq!(workspace.id(), &id);
        assert_eq!(workspace.name(), Some("Acme"));
    }

    #[test]
    fn apply_created_event_to_existing_workspace_returns_already_exists() {
        let id = test_id();
        let existing = Workspace::apply(
            None,
            WorkspaceEvent::Created {
                id: id.clone(),
                name: None,
            },
        )
        .unwrap();
        let result = Workspace::apply(Some(existing), WorkspaceEvent::Created { id, name: None });
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
        let result = Workspace::apply(
            None,
            WorkspaceEvent::UserRoleAssigned {
                user_id,
                workspace_role_id: role_id,
            },
        );
        assert!(matches!(result, Err(Error::NotFound)));
    }
}
