use std::fmt::Debug;

use eventually::aggregate;

use crate::tenant::activity::{
    self,
    domain::{
        aggregates::{Activity, ActivityId},
        events::ActivityEvent,
    },
};

pub trait ActivityCommandTrait<T> {
    type Error: Debug + Sync + Send;

    fn create(
        &self,
        id: ActivityId,
        name: String,
        comment: Option<String>,
    ) -> Result<T, Self::Error>;
}

#[eventually_macros::aggregate_root(Activity)]
pub struct ActivityCommand;

impl ActivityCommandTrait<ActivityCommand> for ActivityCommand {
    type Error = activity::Error;

    fn create(
        &self,
        id: ActivityId,
        name: String,
        comment: Option<String>,
    ) -> Result<ActivityCommand, Self::Error> {
        Ok(aggregate::Root::<Activity>::record_new(
            ActivityEvent::Created { id, name, comment }.into(),
        )?
        .into())
    }
}

impl ActivityCommand {
    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn create(
        id: ActivityId,
        name: String,
        comment: Option<String>,
    ) -> Result<Self, activity::Error> {
        Ok(aggregate::Root::<Activity>::record_new(
            ActivityEvent::Created { id, name, comment }.into(),
        )?
        .into())
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn update(&mut self, name: String, comment: Option<String>) -> Result<(), activity::Error> {
        self.record_that(ActivityEvent::Updated { name, comment }.into())
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn delete(&mut self) -> Result<(), activity::Error> {
        self.record_that(ActivityEvent::Deleted {}.into())
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
                comment: None,
            },
        )
        .expect("seed activity");
        Root::<Activity>::rehydrate_from_state(1, activity).into()
    }

    #[test]
    fn create_returns_root_with_applied_state() {
        let id: ActivityId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92".parse().unwrap();

        let result = ActivityCommand::create(id.clone(), "Stand-up".to_string(), None);

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.aggregate_id(), &id);
        assert_eq!(cmd.name(), "Stand-up");
        assert!(cmd.comment().is_none());
        assert_eq!(cmd.version(), 1);
    }

    #[test]
    fn create_stores_comment_when_provided() {
        let id: ActivityId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92".parse().unwrap();

        let cmd = ActivityCommand::create(
            id,
            "Debug session".to_string(),
            Some("detailed".to_string()),
        )
        .unwrap();
        assert_eq!(cmd.comment(), Some(&"detailed".to_string()));
    }

    #[test]
    fn update_mutates_state_and_increments_version() {
        let id = test_id();
        let mut cmd = make_shell(id);

        cmd.update("Renamed".to_string(), Some("a note".to_string()))
            .expect("update must succeed");

        assert_eq!(cmd.version(), 2);
        assert_eq!(cmd.name(), "Renamed");
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
                comment: Some("original".to_string()),
            },
        )
        .expect("seed activity");
        let mut cmd: ActivityCommand = Root::<Activity>::rehydrate_from_state(1, activity).into();

        cmd.update("Same".to_string(), None)
            .expect("update must succeed");

        assert!(cmd.comment().is_none());
    }
}
