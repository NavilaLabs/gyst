use chrono::{DateTime, Utc};
use eventually::message::Message;
use serde::{Deserialize, Serialize};

use crate::admin::{invitation, user, workspace, workspace_role};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    /// A new invitation was sent.
    Created {
        id: invitation::Id,
        workspace_id: workspace::Id,
        invited_by: user::Id,
        email: String,
        workspace_role_id: workspace_role::Id,
        token: String,
        expires_at: DateTime<Utc>,
    },
    /// The invitation was accepted by the invitee.
    Accepted { accepted_by: user::Id },
    /// The invitation was revoked by the inviter (normally a workspace admin).
    Revoked {},
}

impl Message for Event {
    fn name(&self) -> &'static str {
        match self {
            Self::Created { .. } => "InvitationCreated",
            Self::Accepted { .. } => "InvitationAccepted",
            Self::Revoked {} => "InvitationRevoked",
        }
    }
}
