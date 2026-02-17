use domain::{
    AggregateMeta, EventContext, EventEnvelope, EventTimestamps, EventType, EventVersion,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value as JsonValue;
use sqlx::types::Uuid;

pub mod config;
pub mod database;
pub mod event_store;
pub mod projections;

mod integrity;
pub use integrity::*;

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
    event_id: Uuid,
    event_type: String,
    event_version: u8,

    #[serde(flatten)]
    aggregate: AggregateMeta,
    #[serde(flatten)]
    context: EventContext,
    #[serde(flatten)]
    timestamps: EventTimestamps,

    // Data payloads
    data: JsonValue,
    metadata: Option<JsonValue>,

    // Integrity
    hash: Vec<u8>,
    previous_hash: Vec<u8>,
}

impl EventRecord {
    pub fn get_event_id(&self) -> &Uuid {
        &self.event_id
    }

    pub fn get_event_type(&self) -> &str {
        &self.event_type
    }

    pub fn get_event_version(&self) -> u8 {
        self.event_version
    }

    pub fn get_aggregate(&self) -> &AggregateMeta {
        &self.aggregate
    }

    pub fn get_context(&self) -> &EventContext {
        &self.context
    }

    pub fn get_timestamps(&self) -> &EventTimestamps {
        &self.timestamps
    }

    pub fn get_data(&self) -> &JsonValue {
        &self.data
    }

    pub fn get_metadata(&self) -> &Option<JsonValue> {
        &self.metadata
    }

    pub fn get_hash(&self) -> &Vec<u8> {
        &self.hash
    }

    pub fn get_previous_hash(&self) -> &Vec<u8> {
        &self.previous_hash
    }
}

impl<T: Serialize + EventType + EventVersion> TryFrom<EventEnvelope<T>> for EventRecord {
    type Error = Error;

    fn try_from(event: EventEnvelope<T>) -> Result<Self, Self::Error> {
        let payload = event.get_payload();
        let data = serde_json::to_value(&payload)?;

        let mut record = EventRecord {
            event_id: event.get_event_id().clone(),
            event_type: payload.get_event_type().to_string(),
            event_version: <T as EventVersion>::VERSION,
            aggregate: event.get_aggregate().clone(),
            context: event.get_context().clone(),
            timestamps: event.get_timestamps().clone(),
            data,
            metadata: event.get_metadata().clone(),
            previous_hash: event.get_hash().clone(),
            hash: Vec::new(),
        };
        record.hash = record.calculate_hash(&event.get_hash());

        Ok(record)
    }
}

impl<T: DeserializeOwned> TryInto<EventEnvelope<T>> for EventRecord {
    type Error = Error;

    fn try_into(self) -> Result<EventEnvelope<T>, Self::Error> {
        let payload = serde_json::from_value(self.data)?;

        Ok(EventEnvelope::new(
            self.event_id,
            self.aggregate,
            self.context,
            self.timestamps,
            payload,
            self.metadata,
        ))
    }
}
