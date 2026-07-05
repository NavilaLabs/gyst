use chrono::{DateTime, Utc};
use eventually::message::Message;
use serde::{Deserialize, Serialize, de};

use crate::admin::{user::UserId, workspace::WorkspaceId, workspace_role::WorkspaceRoleId};

use super::aggregates::InvitationId;

// Private helper for derived deserialization of the canonical event format.
// Old "Revoked" events were stored as the bare JSON string "Revoked" (unit variant default).
// Using a separate type avoids the infinite recursion that a self-referential custom
// Deserialize impl would produce when calling serde_json::from_value::<Self>.
#[derive(Deserialize)]
enum InvitationEventDe {
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
    Revoked {},
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
    Revoked {},
}

impl<'de> Deserialize<'de> for InvitationEvent {
    fn deserialize<D: de::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = serde_json::Value::deserialize(d)?;
        // Legacy format: unit variant stored as a bare JSON string "Revoked"
        if v == serde_json::Value::String("Revoked".to_owned()) {
            return Ok(Self::Revoked {});
        }
        let inner: InvitationEventDe = serde_json::from_value(v).map_err(de::Error::custom)?;
        Ok(match inner {
            InvitationEventDe::Created {
                id,
                workspace_id,
                invited_by,
                email,
                workspace_role_id,
                token,
                expires_at,
            } => Self::Created {
                id,
                workspace_id,
                invited_by,
                email,
                workspace_role_id,
                token,
                expires_at,
            },
            InvitationEventDe::Accepted { accepted_by } => Self::Accepted { accepted_by },
            InvitationEventDe::Revoked {} => Self::Revoked {},
        })
    }
}

impl Message for InvitationEvent {
    fn name(&self) -> &'static str {
        match self {
            Self::Created { .. } => "InvitationCreated",
            Self::Accepted { .. } => "InvitationAccepted",
            Self::Revoked {} => "InvitationRevoked",
        }
    }
}
