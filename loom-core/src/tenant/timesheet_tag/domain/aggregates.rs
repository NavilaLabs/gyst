use eventually::aggregate::Aggregate;
use serde::{Deserialize, Serialize};

use crate::shared::AggregateId;
use crate::tenant::timesheet_tag::TimesheetTagEvent;

pub type TimesheetTagId = AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimesheetTag {
    id: TimesheetTagId,
    name: String,
}

impl TimesheetTag {
    #[must_use]
    pub const fn id(&self) -> &TimesheetTagId {
        &self.id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

crate::aggregate_errors!("tag");

impl Aggregate for TimesheetTag {
    type Id = TimesheetTagId;
    type Event = TimesheetTagEvent;
    type Error = Error;

    fn type_name() -> &'static str {
        "timesheet_tag"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match (state, event) {
            (None, TimesheetTagEvent::Created { id, name }) => Ok(Self { id, name }),
            (Some(_), TimesheetTagEvent::Created { .. }) => Err(Error::AlreadyExists),
            (None, _) => Err(Error::NotFound),
            (Some(mut t), TimesheetTagEvent::Renamed { name }) => {
                t.name = name;
                Ok(t)
            }
            (
                Some(t),
                TimesheetTagEvent::TimesheetTagged { .. }
                | TimesheetTagEvent::TimesheetUntagged { .. },
            ) => Ok(t),
        }
    }
}
