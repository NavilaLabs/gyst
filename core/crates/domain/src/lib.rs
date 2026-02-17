use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

pub mod shared;
pub mod tenant;
pub mod user;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    UuidError(#[from] uuid::Error),
    #[error("{0}")]
    HexError(#[from] hex::FromHexError),
}

pub trait EventType {
    fn get_event_type(&self) -> &str;
}

pub trait EventVersion {
    const VERSION: u8;
}

/// Represents the aggregate this event belongs to.
/// Useful for enforcing optimistic concurrency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateMeta {
    aggregate_type: String,
    aggregate_id: Uuid,
    aggregate_version: i32,
}

impl AggregateMeta {
    pub fn new(aggregate_type: String, aggregate_id: Uuid, aggregate_version: i32) -> Self {
        AggregateMeta {
            aggregate_type,
            aggregate_id,
            aggregate_version,
        }
    }

    pub fn get_aggregate_type(&self) -> &str {
        &self.aggregate_type
    }

    pub fn get_aggregate_id(&self) -> &Uuid {
        &self.aggregate_id
    }

    pub fn get_aggregate_version(&self) -> i32 {
        self.aggregate_version
    }
}

/// Tracing information for the Event Store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContext {
    correlation_id: Option<Uuid>,
    causation_id: Option<Uuid>,
    created_by: Uuid,
    owned_by: Option<Uuid>,
}

impl EventContext {
    pub fn new(
        correlation_id: Option<Uuid>,
        causation_id: Option<Uuid>,
        created_by: Uuid,
        owned_by: Option<Uuid>,
    ) -> Self {
        EventContext {
            correlation_id,
            causation_id,
            created_by,
            owned_by,
        }
    }

    pub fn get_correlation_id(&self) -> &Option<Uuid> {
        &self.correlation_id
    }

    pub fn get_causation_id(&self) -> &Option<Uuid> {
        &self.causation_id
    }

    pub fn get_created_by(&self) -> &Uuid {
        &self.created_by
    }

    pub fn get_owned_by(&self) -> &Option<Uuid> {
        &self.owned_by
    }
}

/// Timing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTimestamps {
    created_at: DateTime<Utc>,
    effective_at: Option<DateTime<Utc>>,
}

impl EventTimestamps {
    pub fn new(created_at: DateTime<Utc>, effective_at: Option<DateTime<Utc>>) -> Self {
        EventTimestamps {
            created_at,
            effective_at,
        }
    }

    pub fn get_created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn get_effective_at(&self) -> &Option<DateTime<Utc>> {
        &self.effective_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    event_id: Uuid,
    aggregate: AggregateMeta,
    context: EventContext,
    timestamps: EventTimestamps,
    payload: T, // The actual domain event (e.g., UserCreated)
    metadata: Option<JsonValue>,
}

impl<T> EventEnvelope<T> {
    pub fn new(
        event_id: Uuid,
        aggregate: AggregateMeta,
        context: EventContext,
        timestamps: EventTimestamps,
        payload: T,
        metadata: Option<JsonValue>,
    ) -> Self {
        EventEnvelope {
            event_id,
            aggregate,
            context,
            timestamps,
            payload,
            metadata,
        }
    }

    pub fn get_event_id(&self) -> &Uuid {
        &self.event_id
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

    pub fn get_payload(&self) -> &T {
        &self.payload
    }

    pub fn get_metadata(&self) -> &Option<JsonValue> {
        &self.metadata
    }
}
