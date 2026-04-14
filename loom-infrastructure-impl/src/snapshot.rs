//! Generic event-store + snapshot repository wrapper.
//!
//! [`SnapshotRepository`] bundles the `eventually_any` snapshot [`Repository`]
//! together with the pool it was built from, replacing the repeated
//! `{ database, repository }` pair found in every concrete repository struct.

use std::ops::Deref;

use async_trait::async_trait;
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::aggregate::{Aggregate, Root};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use serde::{Serialize, de::DeserializeOwned};

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
}

impl<A, P> SnapshotRepository<A, P>
where
    A: Aggregate + Serialize + DeserializeOwned + Send + Sync,
    A::Id: ToString,
    A::Event: Serialize + DeserializeOwned + Send + Sync + Clone,
    P: AsRef<sqlx::AnyPool>,
{
    /// Build a new repository, running any pending event-store migrations.
    ///
    /// # Errors
    ///
    /// Returns an error if migrations fail.
    pub async fn from_pool(pool: P) -> Result<Self, sqlx::migrate::MigrateError> {
        let store =
            Repository::new(pool.as_ref().clone(), Json::default(), Json::default()).await?;
        Ok(Self { pool, store })
    }

    /// Direct access to the inner event store, useful when callers need
    /// lower-level event-store operations.
    #[must_use]
    pub const fn event_store(&self) -> &Repository<A, Json<A>, Json<A::Event>> {
        &self.store
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
    A: Aggregate + Serialize + DeserializeOwned,
    A::Id: ToString,
    A::Event: Serialize + DeserializeOwned,
    Repository<A, Json<A>, Json<A::Event>>: Saver<A>,
    P: Send + Sync,
{
    async fn save(&self, root: &mut Root<A>) -> Result<(), SaveError> {
        self.store.save(root).await
    }
}
