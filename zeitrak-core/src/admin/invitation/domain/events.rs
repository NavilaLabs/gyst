use chrono::{DateTime, Utc};
use eventually::message::Message;
use serde::{Deserialize, Serialize};

use crate::admin::{user::UserId, workspace::WorkspaceId, workspace_role::WorkspaceRoleId};

use super::aggregates::InvitationId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvitationEvent {
    Created {
        id: InvitationId,
        workspace_id: WorkspaceId,
        invited_by: UserId,
        email: String,
        workspace_role_id: WorkspaceRoleId,
        token: String,
        expires_at: DateTime<Utc>,
    },
    Accepted {
        accepted_by: UserId,
    },
    Revoked,
}

impl Message for InvitationEvent {
    fn name(&self) -> &'static str {
        match self {
            Self::Created { .. } => "InvitationCreated",
            Self::Accepted { .. } => "InvitationAccepted",
            Self::Revoked => "InvitationRevoked",
        }
    }
}
