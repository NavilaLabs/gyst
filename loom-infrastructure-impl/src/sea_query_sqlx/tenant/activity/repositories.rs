use std::{ops::Deref, str::FromStr};

use async_trait::async_trait;
use eventually::aggregate::Root;
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use loom_core::tenant::activity::{
    Activity, ActivityEvent, ActivityId, ActivityRepository as ActivityRepositoryTrait, ActivityRow,
};
use sqlx::{Row, any::AnyRow};

use crate::{ConnectedTenantPool, snapshot::SnapshotRepository};

pub struct ActivityRepository {
    store: SnapshotRepository<Activity, ConnectedTenantPool>,
}

impl Deref for ActivityRepository {
    type Target = Repository<Activity, Json<Activity>, Json<ActivityEvent>>;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl ActivityRepository {
    /// # Errors
    ///
    /// Returns an error if the event store repository cannot be initialized.
    pub async fn from_pool(pool: ConnectedTenantPool) -> Result<Self, sqlx::migrate::MigrateError> {
        Ok(Self {
            store: SnapshotRepository::from_pool(pool).await?,
        })
    }

    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn all(&self) -> Result<Vec<ActivityRow>, crate::Error> {
        let rows =
            sqlx::query("SELECT id, name, comment FROM projections__activities ORDER BY name")
                .fetch_all(self.store.pool.as_ref())
                .await?;
        rows.into_iter().map(|r| Self::map_row(&r)).collect()
    }

    fn map_row(row: &AnyRow) -> Result<ActivityRow, crate::Error> {
        Ok(ActivityRow::new(
            ActivityId::from_str(&row.try_get::<String, _>("id")?)?,
            row.try_get("name")?,
            row.try_get("comment")?,
        ))
    }
}

#[async_trait]
impl Getter<Activity> for ActivityRepository {
    async fn get(&self, id: &ActivityId) -> Result<Root<Activity>, GetError> {
        self.store.get(id).await
    }
}

#[async_trait]
impl Saver<Activity> for ActivityRepository {
    async fn save(&self, root: &mut Root<Activity>) -> Result<(), SaveError> {
        self.store.save(root).await
    }
}

impl ActivityRepositoryTrait for ActivityRepository {}
