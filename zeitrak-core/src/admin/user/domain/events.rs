use eventually::message::Message;
use serde::{Deserialize, Serialize, de};

use crate::admin::user::UserId;

// Private helper for derived deserialization of the canonical (new) event format.
// We cannot derive Deserialize on UserEvent directly because old events stored the
// "Verified" unit variant as the bare JSON string "Verified" rather than {"Verified":{}}.
// Using a separate type breaks the infinite-recursion that a custom Deserialize impl
// would otherwise cause when it calls serde_json::from_value::<Self>.
#[derive(Deserialize)]
enum UserEventDe {
    Created {
        id: UserId,
        name: String,
        email: String,
        password: String,
    },
    SettingsUpdated {
        timezone: String,
        date_format: String,
        language: String,
    },
    VerificationRequested {
        token: String,
    },
    Verified {},
    InstanceAdminGranted {},
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum UserEvent {
    Created {
        id: UserId,
        name: String,
        email: String,
        password: String,
    },
    SettingsUpdated {
        timezone: String,
        date_format: String,
        language: String,
    },
    VerificationRequested {
        token: String,
    },
    Verified {},
    /// Marks this user as the sole instance admin.
    InstanceAdminGranted {},
}

impl<'de> Deserialize<'de> for UserEvent {
    fn deserialize<D: de::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = serde_json::Value::deserialize(d)?;
        // Legacy format: unit variant stored as a bare JSON string "Verified"
        if v == serde_json::Value::String("Verified".to_owned()) {
            return Ok(Self::Verified {});
        }
        let inner: UserEventDe = serde_json::from_value(v).map_err(de::Error::custom)?;
        Ok(match inner {
            UserEventDe::Created {
                id,
                name,
                email,
                password,
            } => Self::Created {
                id,
                name,
                email,
                password,
            },
            UserEventDe::SettingsUpdated {
                timezone,
                date_format,
                language,
            } => Self::SettingsUpdated {
                timezone,
                date_format,
                language,
            },
            UserEventDe::VerificationRequested { token } => Self::VerificationRequested { token },
            UserEventDe::Verified {} => Self::Verified {},
            UserEventDe::InstanceAdminGranted {} => Self::InstanceAdminGranted {},
        })
    }
}

impl Message for UserEvent {
    fn name(&self) -> &'static str {
        match self {
            Self::Created { .. } => "UserCreated",
            Self::SettingsUpdated { .. } => "UserSettingsUpdated",
            Self::VerificationRequested { .. } => "UserVerificationRequested",
            Self::Verified {} => "UserVerified",
            Self::InstanceAdminGranted {} => "UserInstanceAdminGranted",
        }
    }
}
