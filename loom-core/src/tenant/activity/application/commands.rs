use eventually::aggregate;

use crate::tenant::activity::{
    self,
    domain::{
        aggregates::{Activity, ActivityId},
        events::ActivityEvent,
    },
};

#[eventually_macros::aggregate_root(Activity)]
pub struct ActivityCommand;

impl ActivityCommand {
    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn create(
        &self,
        id: ActivityId,
        name: String,
        comment: Option<String>,
    ) -> Result<Self, crate::Error> {
        Ok(aggregate::Root::<Activity>::record_new(
            ActivityEvent::Created { id, name, comment }.into(),
        )
        .map_err(activity::DomainError::from)?
        .into())
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn update(&mut self, name: String, comment: Option<String>) -> Result<(), crate::Error> {
        self.record_that(ActivityEvent::Updated { name, comment }.into())
            .map_err(|e| activity::DomainError::AggregateError(e).into())
    }
}
