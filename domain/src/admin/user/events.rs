use eventually::message::Message;
use serde::{Deserialize, Serialize};

use crate::admin::user;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    /// A new user was created.
    Created {
        id: user::Id,
        name: String,
        email: String,
        password: String,
    },
    /// A email verification was requested.
    VerificationRequested { token: String },
    /// A user's email was verified.
    Verified {},
    /// A user was granted to be an admin of the whole zeitrak instance.
    InstanceAdminGranted {},
}

impl Message for Event {
    fn name(&self) -> &'static str {
        match self {
            Self::Created { .. } => "UserCreated",
            Self::VerificationRequested { .. } => "UserVerificationRequested",
            Self::Verified {} => "UserVerified",
            Self::InstanceAdminGranted {} => "UserInstanceAdminGranted",
        }
    }
}
