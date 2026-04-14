use std::ops::Deref;

use async_trait::async_trait;
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use loom_core::tenant::timesheet_tag::{
    TimesheetTag, TimesheetTagEvent, TimesheetTagId,
    TimesheetTagRepository as TimesheetTagRepositoryTrait,
};
use sqlx::{Row, any::AnyRow};

use crate::ConnectedTenantPool;

pub struct TimesheetTagRepository {
    pool: ConnectedTenantPool,
    repository: Repository<TimesheetTag, Json<TimesheetTag>, Json<TimesheetTagEvent>>,
}

impl Deref for TimesheetTagRepository {
    type Target = Repository<TimesheetTag, Json<TimesheetTag>, Json<TimesheetTagEvent>>;
    fn deref(&self) -> &Self::Target {
        &self.repository
    }
}

impl TimesheetTagRepository {
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
    pub async fn all(&self) -> Result<Vec<TimesheetTagRow>, crate::Error> {
        let rows = sqlx::query("SELECT id, name FROM projections__timesheet_tags ORDER BY name")
            .fetch_all(self.pool.as_ref())
            .await?;
        rows.into_iter().map(|r| Self::map_row(&r)).collect()
    }

    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn for_timesheet(
        &self,
        timesheet_id: &str,
    ) -> Result<Vec<TimesheetTagRow>, crate::Error> {
        let rows = sqlx::query(
            "SELECT t.id, t.name \
             FROM projections__tags t \
             JOIN projections__timesheet_tags tt ON tt.tag_id = t.id \
             WHERE tt.timesheet_id = ? \
             ORDER BY t.name",
        )
        .bind(timesheet_id)
        .fetch_all(self.pool.as_ref())
        .await?;
        rows.into_iter().map(|r| Self::map_row(&r)).collect()
    }

    fn map_row(row: &AnyRow) -> Result<TimesheetTagRow, crate::Error> {
        Ok(TimesheetTagRow {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TimesheetTagRow {
    pub id: String,
    pub name: String,
}

#[async_trait]
impl Getter<TimesheetTag> for TimesheetTagRepository {
    async fn get(
        &self,
        id: &TimesheetTagId,
    ) -> Result<eventually::aggregate::Root<TimesheetTag>, GetError> {
        self.repository.get(id).await
    }
}

#[async_trait]
impl Saver<TimesheetTag> for TimesheetTagRepository {
    async fn save(
        &self,
        root: &mut eventually::aggregate::Root<TimesheetTag>,
    ) -> Result<(), SaveError> {
        self.repository.save(root).await
    }
}

impl TimesheetTagRepositoryTrait for TimesheetTagRepository {}
