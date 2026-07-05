use eventually::message::Message;
use serde::{Deserialize, Serialize};

use crate::admin::user;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Created {
        id: user::Id,
        name: String,
        email: String,
        password: String,
    },
    VerificationRequested {
        token: String,
    },
    Verified {},
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
