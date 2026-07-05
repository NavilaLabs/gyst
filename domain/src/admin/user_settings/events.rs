use eventually::message::Message;
use serde::{Deserialize, Serialize};

use crate::admin::{user, user_settings};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Created {
        id: user_settings::Id,
        user_id: user::Id,
        timezone: String,
        date_format: String,
        language: String,
    },
    Updated {
        timezone: String,
        date_format: String,
        language: String,
    },
}

impl Message for Event {
    fn name(&self) -> &'static str {
        match self {
            Self::Created { .. } => "UserSettingsCreated",
            Self::Updated { .. } => "UserSettingsUpdated",
        }
    }
}
