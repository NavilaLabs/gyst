use serde::{Deserialize, Serialize};

use crate::admin::{user, user_settings};

pub type Id = crate::AggregateId;

/// A users' defined personal settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregate {
    id: Id,
    user_id: user::Id,
    timezone: String,
    date_format: String,
    language: String,
}

impl Aggregate {
    /// The id of the user settings.
    #[must_use]
    pub const fn id(&self) -> &Id {
        &self.id
    }

    /// The id of the user these settings apply for.
    #[must_use]
    pub const fn user_id(&self) -> &user::Id {
        &self.user_id
    }

    /// The timezone the user creates events with and calculates other entries
    /// to.
    #[must_use]
    pub fn timezone(&self) -> &str {
        &self.timezone
    }

    /// The format the date and time is displayed to the user.
    #[must_use]
    pub fn date_format(&self) -> &str {
        &self.date_format
    }

    /// The language all the text is displayed to the user.
    #[must_use]
    pub fn language(&self) -> &str {
        &self.language
    }
}

impl eventually::aggregate::Aggregate for Aggregate {
    type Id = Id;
    type Event = user_settings::Event;
    type Error = user_settings::Error;

    fn type_name() -> &'static str {
        "user_settings"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }
    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match (state, event) {
            (
                None,
                user_settings::Event::Created {
                    id,
                    user_id,
                    timezone,
                    date_format,
                    language,
                },
            ) => Ok(Self {
                id,
                user_id,
                timezone,
                date_format,
                language,
            }),
            (Some(_), user_settings::Event::Created { .. }) => {
                Err(user_settings::Error::AlreadyExists)
            }
            (None, _) => Err(user_settings::Error::NotFound),
            (
                Some(mut user),
                user_settings::Event::Updated {
                    timezone,
                    date_format,
                    language,
                },
            ) => {
                user.timezone = timezone;
                user.date_format = date_format;
                user.language = language;
                Ok(user)
            }
        }
    }
}

impl crate::snapshot_policy::SnapshotPolicy for Aggregate {}
