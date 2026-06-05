use eventually::aggregate::Aggregate;
use serde::{Deserialize, Serialize};

use crate::{
    admin::user::{self, UserEvent},
    shared::AggregateId,
};

pub type UserId = AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    id: UserId,
    name: String,
    email: String,
    password: String,
    pub timezone: String,
    pub date_format: String,
    pub language: String,
    pub is_verified: bool,
    pub verification_token: Option<String>,
}

impl User {
    #[must_use]
    pub const fn id(&self) -> &UserId {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }
}

impl Aggregate for User {
    type Id = UserId;
    type Event = UserEvent;
    type Error = user::Error;

    fn type_name() -> &'static str {
        "user"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match (state, event) {
            (
                None,
                UserEvent::Created {
                    id,
                    name,
                    email,
                    password,
                },
            ) => Ok(Self {
                id,
                name,
                email,
                password,
                timezone: "Europe/Berlin".to_string(),
                date_format: "%Y-%m-%d".to_string(),
                language: "en".to_string(),
                is_verified: false,
                verification_token: None,
            }),
            (Some(_), UserEvent::Created { .. }) => Err(user::Error::AlreadyExists),
            (None, _) => Err(user::Error::NotFound),
            (
                Some(mut user),
                UserEvent::SettingsUpdated {
                    timezone,
                    date_format,
                    language,
                },
            ) => {
                user.timezone = timezone;
                user.date_format = date_format;
                user.language = language;
                Ok(user)
            }
            (Some(mut user), UserEvent::VerificationRequested { token }) => {
                user.verification_token = Some(token);
                Ok(user)
            }
            (Some(mut user), UserEvent::Verified {}) => {
                user.is_verified = true;
                user.verification_token = None;
                Ok(user)
            }
        }
    }
}

impl crate::snapshot_policy::SnapshotPolicy for User {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::user;

    fn test_id() -> UserId {
        "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
            .parse()
            .expect("valid UUID")
    }

    fn created_event(id: UserId, name: &str) -> UserEvent {
        UserEvent::Created {
            id,
            name: name.to_string(),
            email: "alice@example.com".to_string(),
            password: "$2b$12$hash".to_string(),
        }
    }

    #[test]
    fn apply_verification_requested_sets_token() {
        let id = test_id();
        let user = User::apply(None, created_event(id, "Alice")).unwrap();
        let result = User::apply(
            Some(user),
            UserEvent::VerificationRequested {
                token: "abc".to_string(),
            },
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().verification_token, Some("abc".to_string()));
    }

    #[test]
    fn apply_verified_sets_is_verified_and_clears_token() {
        let id = test_id();
        let user = User::apply(None, created_event(id, "Alice")).unwrap();
        let user = User::apply(
            Some(user),
            UserEvent::VerificationRequested {
                token: "abc".to_string(),
            },
        )
        .unwrap();
        let result = User::apply(Some(user), UserEvent::Verified {});
        assert!(result.is_ok());
        let user = result.unwrap();
        assert!(user.is_verified);
        assert!(user.verification_token.is_none());
    }

    #[test]
    fn apply_verification_on_none_state_returns_not_found() {
        let result = User::apply(
            None,
            UserEvent::VerificationRequested {
                token: "abc".to_string(),
            },
        );
        assert!(matches!(result, Err(user::Error::NotFound)));
    }

    #[test]
    fn apply_created_event_to_no_state_builds_user() {
        let id = test_id();
        let event = created_event(id.clone(), "Alice");
        let result = User::apply(None, event);
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id(), &id);
        assert_eq!(user.name(), "Alice");
        assert_eq!(user.email(), "alice@example.com");
    }

    #[test]
    fn apply_created_event_to_existing_user_returns_already_exists() {
        let id = test_id();
        let existing = User::apply(None, created_event(id.clone(), "Alice")).unwrap();
        let result = User::apply(Some(existing), created_event(id, "Bob"));
        assert!(matches!(result, Err(user::Error::AlreadyExists)));
    }
}
