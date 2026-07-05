use serde::{Deserialize, Serialize};

use crate::admin::user;

pub type Id = crate::AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregate {
    id: Id,
    user_id: user::Id,
    timezone: String,
    date_format: String,
    language: String,
}

impl Aggregate {
    pub const fn id(&self) -> &Id {
        &self.id
    }

    pub const fn user_id(&self) -> &user::Id {
        &self.user_id
    }

    pub fn timezone(&self) -> &str {
        &self.timezone
    }

    pub fn date_format(&self) -> &str {
        &self.date_format
    }

    pub fn language(&self) -> &str {
        &self.language
    }
}
