use std::{collections::HashMap, ops::Deref, str::FromStr};

use async_trait::async_trait;
use eventually::aggregate::Root;
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use loom_core::tenant::timesheet_tag::{
    TimesheetTag, TimesheetTagEvent, TimesheetTagId,
    TimesheetTagRepository as TimesheetTagRepositoryTrait, TimesheetTagRow,
};
use sqlx::{Row, any::AnyRow};

use crate::{ConnectedTenantPool, snapshot::SnapshotRepository};

pub struct TimesheetTagRepository {
    store: SnapshotRepository<TimesheetTag, ConnectedTenantPool>,
}

impl Deref for TimesheetTagRepository {
    type Target = Repository<TimesheetTag, Json<TimesheetTag>, Json<TimesheetTagEvent>>;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl TimesheetTagRepository {
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
    pub async fn all(&self) -> Result<Vec<TimesheetTagRow>, crate::Error> {
        let rows = sqlx::query("SELECT id, name FROM projections__timesheet_tags ORDER BY name")
            .fetch_all(self.store.pool.as_ref())
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
             FROM projections__timesheet_tags t \
             JOIN projections__timesheet_has_tags tht ON tht.timesheet_tag_id = t.id \
             WHERE tht.timesheet_id = ? \
             ORDER BY t.name",
        )
        .bind(timesheet_id)
        .fetch_all(self.store.pool.as_ref())
        .await?;
        rows.into_iter().map(|r| Self::map_row(&r)).collect()
    }

    /// Returns all tag assignments for the given timesheet IDs as a map of `timesheet_id` → tags.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn for_timesheets_batch(
        &self,
        timesheet_ids: &[&str],
    ) -> Result<HashMap<String, Vec<TimesheetTagRow>>, crate::Error> {
        if timesheet_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let placeholders = vec!["?"; timesheet_ids.len()].join(", ");
        let sql = format!(
            "SELECT tht.timesheet_id, t.id, t.name \
             FROM projections__timesheet_has_tags tht \
             JOIN projections__timesheet_tags t ON t.id = tht.timesheet_tag_id \
             WHERE tht.timesheet_id IN ({placeholders}) \
             ORDER BY t.name"
        );
        let mut q = sqlx::query(&sql);
        for id in timesheet_ids {
            q = q.bind(*id);
        }
        let rows = q.fetch_all(self.store.pool.as_ref()).await?;
        let mut result: HashMap<String, Vec<TimesheetTagRow>> = HashMap::new();
        for row in rows {
            let ts_id: String = row.try_get("timesheet_id")?;
            let tag = Self::map_row(&row)?;
            result.entry(ts_id).or_default().push(tag);
        }
        Ok(result)
    }

    fn map_row(row: &AnyRow) -> Result<TimesheetTagRow, crate::Error> {
        Ok(TimesheetTagRow::new(
            TimesheetTagId::from_str(&row.try_get::<String, _>("id")?)?,
            row.try_get("name")?,
        ))
    }
}

#[async_trait]
impl Getter<TimesheetTag> for TimesheetTagRepository {
    async fn get(&self, id: &TimesheetTagId) -> Result<Root<TimesheetTag>, GetError> {
        self.store.get(id).await
    }
}

#[async_trait]
impl Saver<TimesheetTag> for TimesheetTagRepository {
    async fn save(&self, root: &mut Root<TimesheetTag>) -> Result<(), SaveError> {
        self.store.save(root).await
    }
}

impl TimesheetTagRepositoryTrait for TimesheetTagRepository {}
