use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::admin::{user, workspace, workspace_role};

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

/// An aggregate repesenting an invitation.
///
/// An invitation is used to invite new users (members) to workspaces.
/// Only workspace admins can send invitations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregate {
    /// The id of the invitation.
    id: Id,
    /// The [`User`](crate::admin::user::Aggregate) that issued the invitation.
    /// Usually a workspace admin.
    invited_by: user::Id,
    /// The id of the [`Workspace`](crate::admin::workspace::Aggregate) the
    /// user was invited to.
    workspace_id: workspace::Id,
    /// The id of the
    /// [`WorkspaceRole`](crate::admin::workspace_role::Aggregate) that the
    /// invitee is supposed to get if the invitation gets accepted.
    workspace_role_id: workspace_role::Id,
    /// The email of the invitee the invitation gets send to.
    email: String,
    /// The token used to verify the invitations acceptance.
    token: String,
    /// The status of the invitation.
    ///
    /// See: [`Status`]
    status: Status,
    /// The `DateTime` in `Utc` when the invitation will expire.
    expires_at: DateTime<Utc>,
}

impl Aggregate {
    #[must_use]
    pub const fn id(&self) -> &Id {
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
    pub const fn status(&self) -> &Status {
        &self.status
    }

    #[must_use]
    pub const fn is_pending(&self) -> bool {
        matches!(self.status, Status::Pending)
    }

    #[must_use]
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}
