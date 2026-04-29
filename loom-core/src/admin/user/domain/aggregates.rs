use eventually::aggregate::Aggregate;
use serde::{Deserialize, Serialize};

use crate::{admin::user::{self, UserEvent}, shared::AggregateId};

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

impl Aggregate for User
{
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
            }),
            (Some(_), UserEvent::Created { .. }) | (None, UserEvent::SettingsUpdated { .. }) => {
                Err(user::Error::AlreadyExists)
            }
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(matches!(result, Err(Error::AlreadyExists)));
    }
}
