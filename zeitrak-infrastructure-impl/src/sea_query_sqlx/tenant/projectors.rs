use async_trait::async_trait;
use eventually_projection::{Projector, RawEvent};

use crate::{
    ConnectedTenantPool,
    sea_query_sqlx::tenant::{
        activity::projectors::ActivityProjector, timesheet::projectors::TimesheetProjector,
        timesheet_tag::projectors::TimesheetTagProjector,
    },
};

/// A single projector that dispatches each event to all tenant sub-projectors
/// in a fixed, deterministic order.
///
/// Running all projectors under one [`ProjectionRunner`] with one shared
/// checkpoint guarantees that events are applied sequentially across every
/// projection table, preventing FK race conditions (e.g. a `ProjectCreated`
/// event being applied before the corresponding `CustomerCreated` has been
/// committed).
pub struct TenantProjector {
    activity: ActivityProjector,
    timesheet: TimesheetProjector,
    timesheet_tag: TimesheetTagProjector,
}

impl TenantProjector {
    #[must_use]
    pub fn new(pool: &ConnectedTenantPool) -> Self {
        Self {
            activity: ActivityProjector::new(pool.clone()),
            timesheet: TimesheetProjector::new(pool.clone()),
            timesheet_tag: TimesheetTagProjector::new(pool.clone()),
        }
    }
}

#[async_trait]
impl Projector for TenantProjector {
    type Error = crate::Error;

    async fn handle(&mut self, event: RawEvent) -> Result<(), Self::Error> {
        self.activity.handle(event.clone()).await?;
        self.timesheet.handle(event.clone()).await?;
        self.timesheet_tag.handle(event.clone()).await?;
        Ok(())
    }
}
