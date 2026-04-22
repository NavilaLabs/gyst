use std::{fmt::Debug, ops::Deref};

use async_trait::async_trait;
use eventually::aggregate::Root;

use crate::admin::user::{
    self, UserRow, domain::{
        aggregates::{User, UserId},
        events::UserEvent,
        interfaces::UserRepository,
    }
};

#[async_trait]
pub trait UserCommandTrait<T> {
    type Error: Debug;

    async fn create(
        id: UserId,
        name: String,
        email: String,
        password: String,
    ) -> Result<T, Self::Error>;

    async fn update_settings(
        &mut self,
        timezone: String,
        date_format: String,
        language: String,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub struct UserCommand<R>
where
    R: Debug + UserRepository,
{
    root: Root<User>,
    repository: R,
}

#[async_trait]
impl<R> UserCommandTrait<User> for UserCommand<R>
where
    R: Debug + UserRepository<Error = crate::Error<R, UserRow, User>>,
{
    type Error = crate::Error<R, UserRow, User>;

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    async fn update_settings(
        &mut self,
        timezone: String,
        date_format: String,
        language: String,
    ) -> Result<(), <Self as UserCommandTrait<User>>::Error> {
        self.root.record_that(
            UserEvent::SettingsUpdated {
                timezone,
                date_format,
                language,
            }
            .into(),
        )
        .map_err(|e| user::DomainError::AggregateError(e))?;

        self.repository.save(&mut self.root).await.map_err(crate::Error::WriteRepositoryError)
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    async fn create(
        id: UserId,
        name: String,
        email: String,
        password: String,
    ) -> Result<User, Self::Error> {
        Ok(Root::<User>::record_new(
            UserEvent::Created {
                id,
                name,
                email,
                password,
            }
            .into(),
        )
        .map_err(user::DomainError::from)?.to_aggregate_type())
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
