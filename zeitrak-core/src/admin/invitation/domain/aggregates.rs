use chrono::{DateTime, Utc};
use eventually::aggregate::Aggregate;
use serde::{Deserialize, Serialize};

use crate::{
    admin::{
        invitation::{self, InvitationEvent},
        user::UserId,
        workspace::WorkspaceId,
        workspace_role::WorkspaceRoleId,
    },
    shared::AggregateId,
};

pub type InvitationId = AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invitation {
    pub id: InvitationId,
    pub workspace_id: WorkspaceId,
    pub invited_by: UserId,
    pub email: String,
    pub workspace_role_id: WorkspaceRoleId,
    pub token: String,
    pub status: InvitationStatus,
    pub expires_at: DateTime<Utc>,
}

impl Invitation {
    #[must_use]
    pub const fn id(&self) -> &InvitationId {
        &self.id
    }

    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }

    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    #[must_use]
    pub const fn status(&self) -> &InvitationStatus {
        &self.status
    }

    #[must_use]
    pub const fn is_pending(&self) -> bool {
        matches!(self.status, InvitationStatus::Pending)
    }

    #[must_use]
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

impl Aggregate for Invitation {
    type Id = InvitationId;
    type Event = InvitationEvent;
    type Error = invitation::Error;

    fn type_name() -> &'static str {
        "invitation"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match (state, event) {
            (
                None,
                InvitationEvent::Created {
                    id,
                    workspace_id,
                    invited_by,
                    email,
                    workspace_role_id,
                    token,
                    expires_at,
                },
            ) => Ok(Self {
                id,
                workspace_id,
                invited_by,
                email,
                workspace_role_id,
                token,
                status: InvitationStatus::Pending,
                expires_at,
            }),
            (Some(mut inv), InvitationEvent::Accepted { .. }) => {
                inv.status = InvitationStatus::Accepted;
                Ok(inv)
            }
            (Some(mut inv), InvitationEvent::Revoked {}) => {
                inv.status = InvitationStatus::Revoked;
                Ok(inv)
            }
            (None, _) | (Some(_), InvitationEvent::Created { .. }) => {
                Err(invitation::Error::AlreadyExists)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::invitation;

    fn test_created_event() -> InvitationEvent {
        InvitationEvent::Created {
            id: "019d0ce8-facb-7c90-b9d7-000000000001"
                .parse()
                .expect("valid UUID"),
            workspace_id: "019d0ce8-facb-7c90-b9d7-000000000002"
                .parse()
                .expect("valid UUID"),
            invited_by: "019d0ce8-facb-7c90-b9d7-000000000003"
                .parse()
                .expect("valid UUID"),
            email: "bob@example.com".to_string(),
            workspace_role_id: "019d0ce8-facb-7c90-b9d7-000000000004"
                .parse()
                .expect("valid UUID"),
            token: "test-token".to_string(),
            expires_at: Utc::now() + chrono::Duration::days(7),
        }
    }

    #[test]
    fn apply_created_event_builds_pending_invitation() {
        let inv = Invitation::apply(None, test_created_event()).unwrap();
        assert!(matches!(inv.status, InvitationStatus::Pending));
        assert_eq!(inv.email(), "bob@example.com");
    }

    #[test]
    fn apply_accepted_event_transitions_to_accepted() {
        let inv = Invitation::apply(None, test_created_event()).unwrap();
        let accepted_by = "019d0ce8-facb-7c90-b9d7-000000000005"
            .parse()
            .expect("valid UUID");
        let inv = Invitation::apply(Some(inv), InvitationEvent::Accepted { accepted_by }).unwrap();
        assert!(matches!(inv.status, InvitationStatus::Accepted));
    }

    #[test]
    fn apply_revoked_event_transitions_to_revoked() {
        let inv = Invitation::apply(None, test_created_event()).unwrap();
        let inv = Invitation::apply(Some(inv), InvitationEvent::Revoked {}).unwrap();
        assert!(matches!(inv.status, InvitationStatus::Revoked));
    }

    #[test]
    fn apply_created_on_existing_returns_error() {
        let inv = Invitation::apply(None, test_created_event()).unwrap();
        let result = Invitation::apply(Some(inv), test_created_event());
        assert!(matches!(result, Err(invitation::Error::AlreadyExists)));
    }
}
