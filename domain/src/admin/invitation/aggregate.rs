use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::admin::{invitation, user, workspace, workspace_role};

pub type Id = crate::AggregateId;

/// The status of an [`Invatation`].
///
/// Statuses:
/// - **Pending**: When the [`Invatation`] was sent.
/// - **Accepted**: When the invitee accepeted the [`Invatation`].
/// - **Revoked**: When the workspace admin revoked the [`Invatation`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Pending,
    Accepted,
    Revoked,
}

/// An invitation to a workspace for a new member.
///
/// An invitation is used to invite new users (members) to workspaces.
/// Only workspace admins can send invitations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregate {
    id: Id,
    invited_by: user::Id,
    workspace_id: workspace::Id,
    workspace_role_id: workspace_role::Id,
    email: String,
    token: String,
    status: Status,
    expires_at: DateTime<Utc>,
}

impl Aggregate {
    /// The id of the invitation.
    #[must_use]
    pub const fn id(&self) -> &Id {
        &self.id
    }

    /// The [`User`](crate::admin::user::Aggregate) that issued the invitation.
    /// Usually a workspace admin.
    #[must_use]
    pub const fn invited_by(&self) -> &Id {
        &self.invited_by
    }

    /// The id of the [`Workspace`](crate::admin::workspace::Aggregate) the
    /// user was invited to.
    #[must_use]
    pub const fn workspace_id(&self) -> &Id {
        &self.workspace_id
    }

    /// The id of the
    /// [`WorkspaceRole`](crate::admin::workspace_role::Aggregate) that the
    /// invitee is supposed to get if the invitation gets accepted.
    #[must_use]
    pub const fn workspace_role_id(&self) -> &Id {
        &self.workspace_role_id
    }

    /// The email of the invitee the invitation gets send to.
    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }

    /// The token used to verify the invitations acceptance.
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    /// The status of the invitation.
    ///
    /// See: [`Status`]
    #[must_use]
    pub const fn status(&self) -> &Status {
        &self.status
    }

    /// The `DateTime` in `Utc` when the invitation will expire.
    #[must_use]
    pub const fn expires_at(&self) -> &DateTime<Utc> {
        &self.expires_at
    }

    /// Returns true the invation is [`Status::Pending`].
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        matches!(self.status, Status::Pending)
    }

    /// Returns true if [`chrono::Utc::now`] is after [`Self::expires_at`].
    #[must_use]
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

impl eventually::aggregate::Aggregate for Aggregate {
    type Id = Id;
    type Event = invitation::Event;
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
                invitation::Event::Created {
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
                status: Status::Pending,
                expires_at,
            }),
            (Some(mut inv), invitation::Event::Accepted { .. }) => {
                inv.status = Status::Accepted;
                Ok(inv)
            }
            (Some(mut inv), invitation::Event::Revoked {}) => {
                inv.status = Status::Revoked;
                Ok(inv)
            }
            (None, _) | (Some(_), invitation::Event::Created { .. }) => {
                Err(invitation::Error::AlreadyExists)
            }
        }
    }
}

impl crate::snapshot_policy::SnapshotPolicy for Aggregate {}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use eventually::aggregate::Aggregate;

    use crate::admin::invitation;

    fn test_created_event() -> invitation::Event {
        invitation::Event::Created {
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
        let inv = invitation::Aggregate::apply(None, test_created_event()).unwrap();
        assert!(matches!(inv.status, invitation::Status::Pending));
        assert_eq!(inv.email(), "bob@example.com");
    }

    #[test]
    fn apply_accepted_event_transitions_to_accepted() {
        let inv = invitation::Aggregate::apply(None, test_created_event()).unwrap();
        let accepted_by = "019d0ce8-facb-7c90-b9d7-000000000005"
            .parse()
            .expect("valid UUID");
        let inv =
            invitation::Aggregate::apply(Some(inv), invitation::Event::Accepted { accepted_by })
                .unwrap();
        assert!(matches!(inv.status, invitation::Status::Accepted));
    }

    #[test]
    fn apply_revoked_event_transitions_to_revoked() {
        let inv = invitation::Aggregate::apply(None, test_created_event()).unwrap();
        let inv = invitation::Aggregate::apply(Some(inv), invitation::Event::Revoked {}).unwrap();
        assert!(matches!(inv.status, invitation::Status::Revoked));
    }

    #[test]
    fn apply_created_on_existing_returns_error() {
        let inv = invitation::Aggregate::apply(None, test_created_event()).unwrap();
        let result = invitation::Aggregate::apply(Some(inv), test_created_event());
        assert!(matches!(result, Err(invitation::Error::AlreadyExists)));
    }
}
