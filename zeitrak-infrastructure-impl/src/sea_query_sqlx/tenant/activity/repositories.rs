use std::{ops::Deref, str::FromStr};

use async_trait::async_trait;
use eventually::aggregate::{Aggregate, Root};
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use zeitrak_core::shared::repositories::{ReadRepository, WriteRepository};
use zeitrak_core::tenant::activity::{
    Activity, ActivityEvent, ActivityId, ActivityRepository as ActivityRepositoryTrait, ActivityRow,
};
use sea_query::{Condition, Expr, ExprTrait};
use sqlx::{Row, any::AnyRow};

use crate::{
    ConnectedTenantPool, infrastructure::read_model::SeaQueryReadModel, snapshot::SnapshotRepository,
};

const TABLE: &str = "projections__activities";

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

    const fn read_model(&self) -> SeaQueryReadModel<'_, ConnectedTenantPool> {
        SeaQueryReadModel::new(&self.store.pool, TABLE)
    }

    fn entry_to_row(&self, row: AnyRow) -> Result<Root<Activity>, crate::Error> {
        let id: String = row.try_get("id")?;
        let id = ActivityId::from_str(&id)?;
        let name: String = row.try_get("name")?;
        let comment: Option<String> = row.try_get("comment")?;
        let activity = Activity::apply(None, ActivityEvent::Created { id, name, comment })
            .expect("Created event on None state is infallible");
        Ok(Root::rehydrate_from_state(0, activity))
    }

    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn all(&self) -> Result<Vec<ActivityRow>, crate::Error> {
        let rows = sqlx::query(
            "SELECT id, name, comment FROM projections__activities WHERE deleted_at IS NULL ORDER BY name",
        )
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
impl ReadRepository<Activity> for ActivityRepository {
    type Error = crate::Error;
    type Filter = Condition;

    async fn find(&self, id: ActivityId) -> Result<Option<Root<Activity>>, Self::Error> {
        self.find_by(Condition::all().add(Expr::col("id").eq(id.to_string())))
            .await
    }

    async fn find_by(&self, filter: Condition) -> Result<Option<Root<Activity>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let row = rm.fetch_optional_row(&stmt).await?;
        row.map(|r| self.entry_to_row(r)).transpose()
    }

    async fn find_many(&self, ids: Vec<ActivityId>) -> Result<Vec<Root<Activity>>, crate::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let id_strings: Vec<String> = ids.into_iter().map(|id| id.to_string()).collect();
        self.find_many_by(Condition::all().add(Expr::col("id").is_in(id_strings)))
            .await
    }

    async fn find_many_by(&self, filter: Condition) -> Result<Vec<Root<Activity>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let rows = rm.fetch_all_rows(&stmt).await?;
        rows.into_iter().map(|row| self.entry_to_row(row)).collect()
    }

    async fn all(&self) -> Result<Vec<Root<Activity>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select();
        let rows = rm.fetch_all_rows(&stmt).await?;
        rows.into_iter().map(|row| self.entry_to_row(row)).collect()
    }

    async fn count_by(&self, filter: Condition) -> Result<u64, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select_count().cond_where(filter).to_owned();
        rm.count_rows(&stmt).await
    }

    async fn count(&self) -> Result<u64, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select_count();
        rm.count_rows(&stmt).await
    }
}

#[async_trait]
impl WriteRepository<Activity> for ActivityRepository {
    type Error = crate::Error;
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

impl ActivityRepositoryTrait for ActivityRepository {
    type Error = crate::Error;
}
