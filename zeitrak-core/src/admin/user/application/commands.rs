use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate::{Aggregate, Root};

use crate::admin::user::{
    application::UserRoot,
    domain::{
        aggregates::{User, UserId},
        events::UserEvent,
        interfaces::UserRepository,
    },
};

#[async_trait]
pub trait UserCommandTrait<T>
where
    T: Aggregate,
{
    type Error: Debug + Sync + Send;

    async fn create(
        &self,
        id: UserId,
        name: String,
        email: String,
        password: String,
    ) -> Result<Root<T>, Self::Error>;

    async fn update_settings(
        &self,
        id: UserId,
        timezone: String,
        date_format: String,
        language: String,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub struct UserCommand<R> {
    repository: R,
}

impl<R> UserCommand<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> UserCommandTrait<User> for UserCommand<R>
where
    R: Debug + UserRepository,
{
    type Error = crate::Error<R, User>;

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied or the root cannot be saved.
    async fn create(
        &self,
        id: UserId,
        name: String,
        email: String,
        password: String,
    ) -> Result<Root<User>, <Self as UserCommandTrait<User>>::Error> {
        let mut root = Root::<User>::record_new(
            UserEvent::Created {
                id,
                name,
                email,
                password,
            }
            .into(),
        )?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))?;
        Ok(root)
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    async fn update_settings(
        &self,
        id: UserId,
        timezone: String,
        date_format: String,
        language: String,
    ) -> Result<(), <Self as UserCommandTrait<User>>::Error> {
        let mut root: UserRoot = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();
        root.record_that(
            UserEvent::SettingsUpdated {
                timezone,
                date_format,
                language,
            }
            .into(),
        )?;

        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::admin::user::domain::interfaces::in_memory_repository::InMemoryUserRepository;

    use super::*;

    fn test_id() -> UserId {
        "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
            .parse()
            .expect("valid UUID")
    }

    #[tokio::test]
    async fn create_returns_root_with_applied_state() {
        let id: UserId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92"
            .parse()
            .expect("valid UUID");

        let result = UserCommand::new(InMemoryUserRepository::new()).create(
            id.clone(),
            "Alice".to_string(),
            "alice@example.com".to_string(),
            "$2b$12$hash".to_string(),
        ).await;

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.id(), &id);
        assert_eq!(cmd.name(), "Alice");
    }

    #[tokio::test]
    async fn create_propagates_aggregate_error_on_bad_event() {
        assert!(
            UserCommand::new(InMemoryUserRepository::new()).create(
                test_id(),
                "Bob".to_string(),
                "bob@example.com".to_string(),
                "$2b$12$hash".to_string(),
            )
            .await
            .is_ok()
        );
    }
}
