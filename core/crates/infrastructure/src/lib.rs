use domain::{AggregateMeta, EventContext, EventEnvelope, EventTimestamps};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value as JsonValue;
use sqlx::types::Uuid;

pub mod config;
pub mod database;
pub mod event_store;
pub mod projections;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    EnvVarError(#[from] dotenvy::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Url(#[from] url::ParseError),
    #[error("{0}")]
    YamlError(#[from] serde_yaml::Error),
    #[error("{0}")]
    JsonError(#[from] serde_json::Error),

    #[error("{0}")]
    ConfigError(#[from] config::Error),
    #[cfg(feature = "sea-query-sqlx")]
    #[error("{0}")]
    SeaQuerySQLx(#[from] database::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    // Event Identity
    pub event_id: Uuid,
    pub event_type: String,
    pub event_version: u32,

    #[serde(flatten)]
    pub aggregate: AggregateMeta,
    #[serde(flatten)]
    pub context: EventContext,
    #[serde(flatten)]
    pub timestamps: EventTimestamps,

    // Data payloads
    pub data: JsonValue,
    pub metadata: Option<JsonValue>,

    // Integrity
    pub hash: Vec<u8>,
}

impl<T: Serialize> TryFrom<EventEnvelope<T>> for EventRecord {
    type Error = Error;

    fn try_from(event: EventEnvelope<T>) -> Result<Self, Self::Error> {
        let data = serde_json::to_value(&event.get_payload())?;

        Ok(EventRecord {
            event_id: event.get_event_id().clone(),
            event_type: event.get_event_type().clone(),
            event_version: event.get_event_version().clone(),
            aggregate: event.get_aggregate().clone(),
            context: event.get_context().clone(),
            timestamps: event.get_timestamps().clone(),
            data,
            metadata: event.get_metadata().clone(),
            hash: event.get_hash().clone(),
        })
    }
}

impl<T: DeserializeOwned> TryInto<EventEnvelope<T>> for EventRecord {
    type Error = Error;

    fn try_into(self) -> Result<EventEnvelope<T>, Self::Error> {
        let payload = serde_json::from_value(self.data)?;

        Ok(EventEnvelope::new(
            self.event_id,
            self.event_type,
            self.event_version,
            self.aggregate,
            self.context,
            self.timestamps,
            payload,
            self.metadata,
            self.hash,
        ))
    }
}
