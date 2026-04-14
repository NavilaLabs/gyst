use eventually::aggregate::Aggregate;
use serde::{Deserialize, Serialize};

use crate::shared::AggregateId;
use crate::tenant::activity::ActivityEvent;

pub type ActivityId = AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Activity {
    id: ActivityId,
    name: String,
    comment: Option<String>,
}

impl Activity {
    #[must_use]
    pub const fn id(&self) -> &ActivityId {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn comment(&self) -> Option<&String> {
        self.comment.as_ref()
    }
}

crate::aggregate_errors!("activity");

impl Aggregate for Activity {
    type Id = ActivityId;
    type Event = ActivityEvent;
    type Error = Error;

    fn type_name() -> &'static str {
        "activity"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match (state, event) {
            (None, ActivityEvent::Created { id, name, comment }) => Ok(Self { id, name, comment }),
            (Some(_), ActivityEvent::Created { .. }) => Err(Error::AlreadyExists),
            (None, _) => Err(Error::NotFound),
            (Some(mut a), ActivityEvent::Updated { name, comment, .. }) => {
                a.name = name;
                a.comment = comment;
                Ok(a)
            }
        }
    }
}
