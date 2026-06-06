//! Generic event-store + snapshot repository wrapper.
//!
//! [`SnapshotRepository`] bundles the `eventually_any` snapshot [`Repository`]
//! together with the pool it was built from, replacing the repeated
//! `{ database, repository }` pair found in every concrete repository struct.

use std::ops::Deref;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::aggregate::{Aggregate, Root};
use eventually::message::Message as _;
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use eventually_any::upcasting::{UpcasterChain, Upcaster};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use zeitrak_core::event_upcaster::{EventUpcaster, UpcastError};
use zeitrak_core::shared::event_bus::{DomainEventEnvelope, DomainEventHandler};
use zeitrak_core::snapshot_policy::SnapshotPolicy;

/// Bundles a snapshot-capable event-store repository with the pool it was
/// constructed from.
///
/// Every aggregate repository that uses `eventually_any::snapshot::Repository`
/// previously had two fields (`pool` / `database` + `repository`) and
/// identical boilerplate for `from_pool`, `Deref`, `event_store`, `Getter`,
/// and `Saver`. This type eliminates that repetition.
///
/// # Type parameters
///
/// - `A` — the aggregate type (must implement [`Aggregate`])
/// - `P` — the pool type (e.g. [`crate::ConnectedAdminPool`] or
///   [`crate::ConnectedTenantPool`])
pub struct SnapshotRepository<A, P>
where
    A: Aggregate + Serialize + DeserializeOwned,
    A::Id: ToString,
    A::Event: Serialize + DeserializeOwned,
{
    /// The underlying pool — exposed as `pub` so concrete repositories can
    /// access it for bespoke projection queries without an extra accessor.
    pub pool: P,
    store: Repository<A, Json<A>, Json<A::Event>>,
    /// Optional handler that receives each saved event as a `DomainEventEnvelope`
    /// after a successful aggregate save.  Errors are logged and discarded so
    /// they never cause a save failure.
    event_publisher: Option<Arc<dyn DomainEventHandler>>,
}

impl<A, P> SnapshotRepository<A, P>
where
    A: Aggregate + Serialize + DeserializeOwned + Send + Sync + SnapshotPolicy,
    A::Id: ToString,
    A::Event: Serialize + DeserializeOwned + Send + Sync + Clone,
    P: AsRef<sqlx::AnyPool>,
{
    /// Build a new repository, running any pending event-store migrations.
    ///
    /// The snapshot interval is taken from [`SnapshotPolicy::SNAPSHOT_EVERY`] on `A`.
    ///
    /// # Errors
    ///
    /// Returns an error if migrations fail.
    pub async fn from_pool(pool: P) -> Result<Self, sqlx::migrate::MigrateError> {
        let store = Repository::new(pool.as_ref().clone(), Json::default(), Json::default())
            .await?
            .with_snapshot_every(A::SNAPSHOT_EVERY as usize);
        Ok(Self {
            pool,
            store,
            event_publisher: None,
        })
    }

    /// Attach an event publisher that is called after every successful save.
    ///
    /// Errors returned by the publisher are logged and discarded — they do not
    /// cause the save to fail.
    #[must_use]
    pub fn with_event_publisher(mut self, publisher: Arc<dyn DomainEventHandler>) -> Self {
        self.event_publisher = Some(publisher);
        self
    }

    /// Build a new repository with an [`UpcasterChain`] applied at event read time.
    ///
    /// `upcasters` are applied in registration order; register lower `source_version`
    /// upcasters first. `schema_version` is stamped on every newly written event.
    /// The snapshot interval is taken from [`SnapshotPolicy::SNAPSHOT_EVERY`] on `A`.
    ///
    /// # Errors
    ///
    /// Returns an error if migrations fail.
    pub async fn from_pool_with_upcasters(
        pool: P,
        upcasters: Vec<Arc<dyn EventUpcaster>>,
        schema_version: u32,
    ) -> Result<Self, sqlx::migrate::MigrateError> {
        let chain = upcasters
            .into_iter()
            .fold(UpcasterChain::new(), |c, u| c.register(ZeitrakUpcasterBridge(u)));
        let store = Repository::new(pool.as_ref().clone(), Json::default(), Json::default())
            .await?
            .with_upcaster_chain(chain)
            .with_schema_version(schema_version)
            .with_snapshot_every(A::SNAPSHOT_EVERY as usize);
        Ok(Self {
            pool,
            store,
            event_publisher: None,
        })
    }

    /// Direct access to the inner event store, useful when callers need
    /// lower-level event-store operations.
    #[must_use]
    pub const fn event_store(&self) -> &Repository<A, Json<A>, Json<A::Event>> {
        &self.store
    }
}

// ── Upcaster bridge ───────────────────────────────────────────────────────────

/// Adapts a [`zeitrak_core::event_upcaster::EventUpcaster`] to the
/// `eventually_any` [`Upcaster`] contract, which is infallible.
///
/// On upcast failure the error is logged and `Value::Null` is returned so the
/// caller receives a clear JSON deserialisation error rather than silently
/// incorrect data.
struct ZeitrakUpcasterBridge(Arc<dyn EventUpcaster>);

impl Upcaster for ZeitrakUpcasterBridge {
    fn event_type(&self) -> &str {
        self.0.event_type()
    }

    fn from_version(&self) -> u32 {
        self.0.source_version()
    }

    fn to_version(&self) -> u32 {
        self.0.target_version()
    }

    fn upcast(&self, payload: Value) -> Value {
        self.0.upcast(payload).unwrap_or_else(|e: UpcastError| {
            tracing::error!(
                event_type = self.0.event_type(),
                source_version = self.0.source_version(),
                error = %e,
                "upcaster failed; returning null to surface as a deserialisation error"
            );
            Value::Null
        })
    }
}

impl<A, P> Deref for SnapshotRepository<A, P>
where
    A: Aggregate + Serialize + DeserializeOwned,
    A::Id: ToString,
    A::Event: Serialize + DeserializeOwned,
{
    type Target = Repository<A, Json<A>, Json<A::Event>>;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

#[async_trait]
impl<A, P> Getter<A> for SnapshotRepository<A, P>
where
    A: Aggregate + Serialize + DeserializeOwned,
    A::Id: ToString,
    A::Event: Serialize + DeserializeOwned,
    Repository<A, Json<A>, Json<A::Event>>: Getter<A>,
    P: Send + Sync,
{
    async fn get(&self, id: &A::Id) -> Result<Root<A>, GetError> {
        self.store.get(id).await
    }
}

#[async_trait]
impl<A, P> Saver<A> for SnapshotRepository<A, P>
where
    A: Aggregate + Clone + Serialize + DeserializeOwned + Send + Sync,
    A::Id: ToString + Send + Sync,
    A::Event: Serialize + DeserializeOwned + Send + Sync + Clone,
    Repository<A, Json<A>, Json<A::Event>>: Saver<A>,
    P: Send + Sync,
{
    async fn save(&self, root: &mut Root<A>) -> Result<(), SaveError> {
        // Peek at pending events before the inner save drains them from root.
        // Clone root only when a publisher is attached to avoid unnecessary work.
        let pending = if self.event_publisher.is_some() {
            root.clone().take_uncommitted_events()
        } else {
            vec![]
        };

        let aggregate_id = root.aggregate_id().to_string();
        self.store.save(root).await?;

        if let Some(publisher) = &self.event_publisher {
            for event in pending {
                let envelope = DomainEventEnvelope {
                    aggregate_type: A::type_name(),
                    aggregate_id: aggregate_id.clone(),
                    event_name: event.message.name(),
                    payload: serde_json::to_value(&event.message).unwrap_or_default(),
                    occurred_at: Utc::now(),
                };
                if let Err(e) = publisher.on_event(&envelope).await {
                    tracing::warn!(
                        aggregate_type = A::type_name(),
                        aggregate_id = %aggregate_id,
                        event_name = event.message.name(),
                        error = %e,
                        "event publisher failed after save — discarding"
                    );
                }
            }
        }

        Ok(())
    }
}
