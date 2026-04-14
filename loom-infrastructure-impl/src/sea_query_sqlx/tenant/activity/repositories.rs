use std::ops::Deref;
use std::str::FromStr;

use async_trait::async_trait;
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use loom_core::tenant::activity::{
    Activity, ActivityEvent, ActivityId, ActivityRepository as ActivityRepositoryTrait,
    ActivityView,
};
use sqlx::{Row, any::AnyRow};

use crate::ConnectedTenantPool;

pub struct ActivityRepository {
    pool: ConnectedTenantPool,
    repository: Repository<Activity, Json<Activity>, Json<ActivityEvent>>,
}

impl Deref for ActivityRepository {
    type Target = Repository<Activity, Json<Activity>, Json<ActivityEvent>>;
    fn deref(&self) -> &Self::Target {
        &self.repository
    }
}

impl ActivityRepository {
    /// # Errors
    ///
    /// Returns an error if the event store repository cannot be initialized.
    pub async fn from_pool(pool: ConnectedTenantPool) -> Result<Self, sqlx::migrate::MigrateError> {
        let repository =
            Repository::new(pool.as_ref().clone(), Json::default(), Json::default()).await?;
        Ok(Self { pool, repository })
    }

    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn all(&self) -> Result<Vec<ActivityView>, crate::Error> {
        let rows = sqlx::query(
            "SELECT id, project_id, name, comment, visible, billable \
             FROM projections__activities ORDER BY name",
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        rows.into_iter().map(|r| Self::map_row(&r)).collect()
    }

    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn by_project(&self, project_id: &str) -> Result<Vec<ActivityView>, crate::Error> {
        let rows = sqlx::query(
            "SELECT id, project_id, name, comment, visible, billable \
             FROM projections__activities WHERE project_id = ? OR project_id IS NULL ORDER BY name",
        )
        .bind(project_id)
        .fetch_all(self.pool.as_ref())
        .await?;
        rows.into_iter().map(|r| Self::map_row(&r)).collect()
    }

    fn map_row(row: &AnyRow) -> Result<ActivityView, crate::Error> {
        Ok(ActivityView::new(
            ActivityId::from_str(&row.try_get::<String, _>("id")?)?,
            row.try_get("name")?,
            row.try_get("comment")?,
        ))
    }
}

#[async_trait]
impl Getter<Activity> for ActivityRepository {
    async fn get(
        &self,
        id: &ActivityId,
    ) -> Result<eventually::aggregate::Root<Activity>, GetError> {
        self.repository.get(id).await
    }
}

#[async_trait]
impl Saver<Activity> for ActivityRepository {
    async fn save(
        &self,
        root: &mut eventually::aggregate::Root<Activity>,
    ) -> Result<(), SaveError> {
        self.repository.save(root).await
    }
}

impl ActivityRepositoryTrait for ActivityRepository {}
