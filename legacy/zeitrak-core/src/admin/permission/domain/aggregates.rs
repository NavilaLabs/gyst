use eventually::aggregate::Aggregate;
use serde::{Deserialize, Serialize};

use crate::{
    admin::permission::{self, PermissionEvent},
    shared::AggregateId,
};

pub type PermissionId = AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permission {
    id: PermissionId,
    name: String,
}

impl Permission {
    #[must_use]
    pub const fn id(&self) -> &PermissionId {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Aggregate for Permission {
    type Id = PermissionId;
    type Event = PermissionEvent;
    type Error = permission::Error;

    fn type_name() -> &'static str {
        "permission"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match (state, event) {
            (None, PermissionEvent::Created { id, name }) => Ok(Self { id, name }),
            (Some(_), PermissionEvent::Created { .. }) => Err(permission::Error::AlreadyExists),
        }
    }
}

impl crate::snapshot_policy::SnapshotPolicy for Permission {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::permission::Error;

    fn test_id() -> PermissionId {
        "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
            .parse()
            .expect("valid UUID")
    }

    #[test]
    fn apply_created_event_to_no_state_builds_permission() {
        let id = test_id();
        let event = PermissionEvent::Created {
            id: id.clone(),
            name: "can_invite_users".to_string(),
        };
        let result = Permission::apply(None, event);
        assert!(result.is_ok());
        let permission = result.unwrap();
        assert_eq!(permission.id(), &id);
        assert_eq!(permission.name(), "can_invite_users");
    }

    #[test]
    fn apply_created_event_to_existing_permission_returns_already_exists() {
        let id = test_id();
        let existing = Permission::apply(
            None,
            PermissionEvent::Created {
                id: id.clone(),
                name: "can_invite_users".to_string(),
            },
        )
        .unwrap();
        let result = Permission::apply(
            Some(existing),
            PermissionEvent::Created {
                id,
                name: "can_invite_users".to_string(),
            },
        );
        assert!(matches!(result, Err(Error::AlreadyExists)));
    }
}
