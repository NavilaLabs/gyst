use serde::{Deserialize, Serialize};

use crate::admin::user;

pub type Id = crate::AggregateId;

/// A user of zeitrak.
///
/// A user can be a member of multiple workspaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregate {
    id: Id,
    name: String,
    email: String,
    password: String,
    is_verified: bool,
    is_instance_admin: bool,
    verification_token: Option<String>,
    #[serde(default)]
    is_deleted: bool,
}

impl Aggregate {
    /// The id of the user.
    #[must_use]
    pub const fn id(&self) -> &Id {
        &self.id
    }

    /// The name of the user.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The user's email.
    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }

    /// The user's password used for authentication.
    #[must_use]
    pub fn password(&self) -> &str {
        &self.password
    }

    /// Whether the user had verified their email address.
    #[must_use]
    pub const fn is_verified(&self) -> bool {
        self.is_verified
    }

    /// Whether the user is the zeitrak's instance admin. There can only be one
    /// admin for the whole instance. It is the admin with the most rights in
    /// the whole instance. The instance admin has access to all data, so all
    /// workspaces, users, and so on.
    #[must_use]
    pub const fn is_instance_admin(&self) -> bool {
        self.is_instance_admin
    }

    /// The token used to verify the user.
    #[must_use]
    pub fn verification_token(&self) -> Option<&str> {
        self.verification_token.as_deref()
    }

    /// Whether the user is deleted or not.
    #[must_use]
    pub const fn is_deleted(&self) -> bool {
        self.is_deleted
    }
}

impl eventually::aggregate::Aggregate for Aggregate {
    type Id = Id;
    type Event = user::Event;
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
                user::Event::Created {
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
                is_verified: false,
                verification_token: None,
                is_instance_admin: false,
                is_deleted: false,
            }),
            (Some(_), user::Event::Created { .. }) => Err(user::Error::AlreadyExists),
            (None, _) => Err(user::Error::NotFound),
            (Some(mut user), user::Event::VerificationRequested { token }) => {
                user.verification_token = Some(token);
                Ok(user)
            }
            (Some(mut user), user::Event::Verified {}) => {
                user.is_verified = true;
                user.verification_token = None;
                Ok(user)
            }
            (Some(mut user), user::Event::InstanceAdminGranted {}) => {
                user.is_instance_admin = true;
                Ok(user)
            }
        }
    }
}

impl crate::snapshot_policy::SnapshotPolicy for Aggregate {}

#[cfg(test)]
mod tests {
    use eventually::aggregate::Aggregate;

    use super::*;
    use crate::admin::user;

    fn test_id() -> Id {
        "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
            .parse()
            .expect("valid UUID")
    }

    fn created_event(id: Id, name: &str) -> user::Event {
        user::Event::Created {
            id,
            name: name.to_string(),
            email: "alice@example.com".to_string(),
            password: "$2b$12$hash".to_string(),
        }
    }

    #[test]
    fn apply_verification_requested_sets_token() {
        let id = test_id();
        let user = user::Aggregate::apply(None, created_event(id, "Alice")).unwrap();
        let result = user::Aggregate::apply(
            Some(user),
            user::Event::VerificationRequested {
                token: "abc".to_string(),
            },
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().verification_token, Some("abc".to_string()));
    }

    #[test]
    fn apply_verified_sets_is_verified_and_clears_token() {
        let id = test_id();
        let user = user::Aggregate::apply(None, created_event(id, "Alice")).unwrap();
        let user = user::Aggregate::apply(
            Some(user),
            user::Event::VerificationRequested {
                token: "abc".to_string(),
            },
        )
        .unwrap();
        let result = user::Aggregate::apply(Some(user), user::Event::Verified {});
        assert!(result.is_ok());
        let user = result.unwrap();
        assert!(user.is_verified);
        assert!(user.verification_token.is_none());
    }

    #[test]
    fn apply_verification_on_none_state_returns_not_found() {
        let result = user::Aggregate::apply(
            None,
            user::Event::VerificationRequested {
                token: "abc".to_string(),
            },
        );
        assert!(matches!(result, Err(user::Error::NotFound)));
    }

    #[test]
    fn apply_created_event_to_no_state_builds_user() {
        let id = test_id();
        let event = created_event(id.clone(), "Alice");
        let result = user::Aggregate::apply(None, event);
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id(), &id);
        assert_eq!(user.name(), "Alice");
        assert_eq!(user.email(), "alice@example.com");
    }

    #[test]
    fn apply_created_event_to_existing_user_returns_already_exists() {
        let id = test_id();
        let existing = user::Aggregate::apply(None, created_event(id.clone(), "Alice")).unwrap();
        let result = user::Aggregate::apply(Some(existing), created_event(id, "Bob"));
        assert!(matches!(result, Err(user::Error::AlreadyExists)));
    }
}
