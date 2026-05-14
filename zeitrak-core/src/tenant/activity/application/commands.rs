use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate;

use crate::tenant::activity::{
    self,
    application::views::ActivityRow,
    domain::{
        aggregates::{Activity, ActivityId},
        events::ActivityEvent,
        interfaces::ActivityRepository,
    },
};

pub trait ActivityCommandTrait<T> {
    type Error: Debug + Sync + Send;

    /// # Errors
    ///
    /// Returns an error if the event cannot be applied to the aggregate.
    fn create(
        &self,
        id: ActivityId,
        name: String,
        color: String,
        comment: Option<String>,
    ) -> Result<T, Self::Error>;

    /// # Errors
    ///
    /// Returns an error if the event cannot be applied to the aggregate.
    fn update(&mut self, name: String, color: String, comment: Option<String>) -> Result<(), Self::Error>;

    /// # Errors
    ///
    /// Returns an error if the event cannot be applied to the aggregate.
    fn delete(&mut self) -> Result<(), Self::Error>;
}

#[eventually_macros::aggregate_root(Activity)]
pub struct ActivityCommand;

impl ActivityCommandTrait<Self> for ActivityCommand {
    type Error = activity::Error;

    fn create(
        &self,
        id: ActivityId,
        name: String,
        color: String,
        comment: Option<String>,
    ) -> Result<Self, Self::Error> {
        Ok(aggregate::Root::<Activity>::record_new(
            ActivityEvent::Created { id, name, color, comment }.into(),
        )?
        .into())
    }

    fn update(&mut self, name: String, color: String, comment: Option<String>) -> Result<(), Self::Error> {
        self.record_that(ActivityEvent::Updated { name, color, comment }.into())
    }

    fn delete(&mut self) -> Result<(), Self::Error> {
        self.record_that(ActivityEvent::Deleted {}.into())
    }
}

impl ActivityCommand {
    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn create(
        id: ActivityId,
        name: String,
        color: String,
        comment: Option<String>,
    ) -> Result<Self, activity::Error> {
        Ok(aggregate::Root::<Activity>::record_new(
            ActivityEvent::Created { id, name, color, comment }.into(),
        )?
        .into())
    }
}

#[async_trait]
pub trait ActivityHandlerTrait<R> {
    type Error: Debug + Sync + Send;

    async fn create(
        &self,
        id: ActivityId,
        name: String,
        color: String,
        comment: Option<String>,
    ) -> Result<ActivityRow, Self::Error>;

    async fn update(
        &self,
        id: ActivityId,
        name: String,
        color: String,
        comment: Option<String>,
    ) -> Result<(), Self::Error>;

    async fn delete(&self, id: ActivityId) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub struct ActivityHandler<Repo> {
    repository: Repo,
}

impl<Repo> ActivityHandler<Repo> {
    pub const fn new(repository: Repo) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<Repo, R> ActivityHandlerTrait<R> for ActivityHandler<Repo>
where
    Repo: Debug + ActivityRepository<R>,
{
    type Error = crate::Error<Repo, Activity, R>;

    async fn create(
        &self,
        id: ActivityId,
        name: String,
        color: String,
        comment: Option<String>,
    ) -> Result<ActivityRow, Self::Error> {
        let mut root = aggregate::Root::<Activity>::record_new(
            ActivityEvent::Created {
                id: id.clone(),
                name: name.clone(),
                color: color.clone(),
                comment: comment.clone(),
            }
            .into(),
        )?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))?;
        Ok(ActivityRow::new(id, name, color, comment))
    }

    async fn update(
        &self,
        id: ActivityId,
        name: String,
        color: String,
        comment: Option<String>,
    ) -> Result<(), Self::Error> {
        let mut root: ActivityCommand = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();
        root.update(name, color, comment)?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
    }

    async fn delete(&self, id: ActivityId) -> Result<(), Self::Error> {
        let mut root: ActivityCommand = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();
        root.delete()?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
    }
}

#[cfg(test)]
mod tests {
    use eventually::aggregate::{Aggregate, Root};

    use super::*;

    fn test_id() -> ActivityId {
        "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
            .parse()
            .expect("valid UUID")
    }

    fn make_shell(id: ActivityId) -> ActivityCommand {
        let activity = Activity::apply(
            None,
            ActivityEvent::Created {
                id,
                name: "seed".to_string(),
                color: "#22c55e".to_string(),
                comment: None,
            },
        )
        .expect("seed activity");
        Root::<Activity>::rehydrate_from_state(1, activity).into()
    }

    #[test]
    fn create_returns_root_with_applied_state() {
        let id: ActivityId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92".parse().unwrap();

        let result =
            ActivityCommand::create(id.clone(), "Stand-up".to_string(), "#3b82f6".to_string(), None);

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.aggregate_id(), &id);
        assert_eq!(cmd.name(), "Stand-up");
        assert_eq!(cmd.color(), "#3b82f6");
        assert!(cmd.comment().is_none());
        assert_eq!(cmd.version(), 1);
    }

    #[test]
    fn create_stores_comment_when_provided() {
        let id: ActivityId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92".parse().unwrap();

        let cmd = ActivityCommand::create(
            id,
            "Debug session".to_string(),
            "#22c55e".to_string(),
            Some("detailed".to_string()),
        )
        .unwrap();
        assert_eq!(cmd.comment(), Some(&"detailed".to_string()));
    }

    #[test]
    fn update_mutates_state_and_increments_version() {
        let id = test_id();
        let mut cmd = make_shell(id);

        cmd.update("Renamed".to_string(), "#a855f7".to_string(), Some("a note".to_string()))
            .expect("update must succeed");

        assert_eq!(cmd.version(), 2);
        assert_eq!(cmd.name(), "Renamed");
        assert_eq!(cmd.color(), "#a855f7");
        assert_eq!(cmd.comment(), Some(&"a note".to_string()));
    }

    #[test]
    fn update_can_clear_comment() {
        let id = test_id();
        let activity = Activity::apply(
            None,
            ActivityEvent::Created {
                id,
                name: "seed".to_string(),
                color: "#22c55e".to_string(),
                comment: Some("original".to_string()),
            },
        )
        .expect("seed activity");
        let mut cmd: ActivityCommand = Root::<Activity>::rehydrate_from_state(1, activity).into();

        cmd.update("Same".to_string(), "#22c55e".to_string(), None)
            .expect("update must succeed");

        assert!(cmd.comment().is_none());
    }
}
