use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::admin::{user, workspace, workspace_role};

pub type Id = crate::AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Pending,
    Accepted,
    Revoked,
}

/// An aggregate repesenting an invitation.
///
/// An invitation is used to invite new users (members) to workspaces.
/// Only workspace admins can send invitations. When an invitation is
/// sent it has the status [`invitation::Status::Pending`](Status). The
/// invitation can then be accepted by the invitee or can be revoked
/// by the workspace admin. The `status` is then set to the
/// corresponding [`Status`], [`invitation::Status::Accepted`](Status)
/// or [`invitation::Status::Revoked`](Status).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregate {
    id: Id,
    workspace_id: workspace::Id,
    invited_by: user::Id,
    email: String,
    workspace_role_id: workspace_role::Id,
    token: String,
    status: Status,
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
