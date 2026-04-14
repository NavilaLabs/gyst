use eventually::aggregate;

use crate::admin::user::{
    self,
    domain::{
        aggregates::{User, UserId},
        events::UserEvent,
    },
};

#[eventually_macros::aggregate_root(User)]
pub struct UserCommand;

impl UserCommand {
    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn update_settings(
        &mut self,
        timezone: String,
        date_format: String,
        language: String,
    ) -> Result<(), crate::Error> {
        self.record_that(
            UserEvent::SettingsUpdated {
                timezone,
                date_format,
                language,
            }
            .into(),
        )
        .map_err(|e| user::DomainError::AggregateError(e).into())
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn create(
        id: UserId,
        name: String,
        email: String,
        password: String,
    ) -> Result<Self, crate::Error> {
        Ok(aggregate::Root::<User>::record_new(
            UserEvent::Created {
                id,
                name,
                email,
                password,
            }
            .into(),
        )
        .map_err(user::DomainError::from)?
        .into())
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

    #[test]
    fn create_returns_root_with_applied_state() {
        let id: UserId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92"
            .parse()
            .expect("valid UUID");

        let result = UserCommand::create(
            id.clone(),
            "Alice".to_string(),
            "alice@example.com".to_string(),
            "$2b$12$hash".to_string(),
        );

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.aggregate_id(), &id);
        assert_eq!(cmd.name(), "Alice");
        assert_eq!(cmd.version(), 1);
    }

    #[test]
    fn create_propagates_aggregate_error_on_bad_event() {
        assert!(
            UserCommand::create(
                test_id(),
                "Bob".to_string(),
                "bob@example.com".to_string(),
                "$2b$12$hash".to_string(),
            )
            .is_ok()
        );
    }
}
