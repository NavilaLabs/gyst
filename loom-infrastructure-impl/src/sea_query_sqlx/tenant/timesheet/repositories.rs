use std::{ops::Deref, str::FromStr};

use async_trait::async_trait;
use eventually::aggregate::Root;
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use loom_core::admin::user::UserId;
use loom_core::tenant::activity::ActivityId;
use loom_core::tenant::timesheet::{
    Timesheet, TimesheetEvent, TimesheetId, TimesheetRepository as TimesheetRepositoryTrait,
    TimesheetRow,
};
use sqlx::{Row, any::AnyRow};

use crate::{ConnectedTenantPool, snapshot::SnapshotRepository};

pub struct TimesheetRepository {
    store: SnapshotRepository<Timesheet, ConnectedTenantPool>,
}

impl Deref for TimesheetRepository {
    type Target = Repository<Timesheet, Json<Timesheet>, Json<TimesheetEvent>>;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl TimesheetRepository {
    /// # Errors
    ///
    /// Returns an error if the event store repository cannot be initialized.
    pub async fn from_pool(pool: ConnectedTenantPool) -> Result<Self, sqlx::migrate::MigrateError> {
        Ok(Self {
            store: SnapshotRepository::from_pool(pool).await?,
        })
    }

    const SELECT: &'static str = "SELECT id, user_id, activity_id, start_time, end_time, duration, description, timezone \
         FROM projections__timesheets";

    /// Most-recent 50 non-cancelled timesheets for a user, newest first.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn recent_for_user(&self, user_id: &str) -> Result<Vec<TimesheetRow>, crate::Error> {
        let sql = format!(
            "{} WHERE user_id = ? AND cancelled_at IS NULL ORDER BY start_time DESC LIMIT 50",
            Self::SELECT
        );
        let rows = sqlx::query(&sql)
            .bind(user_id)
            .fetch_all(self.store.pool.as_ref())
            .await?;
        rows.into_iter().map(|r| Self::map_row(&r)).collect()
    }

    /// Returns the running timesheet for a user (`end_time` IS NULL), if any.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn running_for_user(
        &self,
        user_id: &str,
    ) -> Result<Option<TimesheetRow>, crate::Error> {
        let sql = format!(
            "{} WHERE user_id = ? AND end_time IS NULL AND cancelled_at IS NULL ORDER BY start_time DESC LIMIT 1",
            Self::SELECT
        );
        let row = sqlx::query(&sql)
            .bind(user_id)
            .fetch_optional(self.store.pool.as_ref())
            .await?;
        row.map(|r| Self::map_row(&r)).transpose()
    }

    fn map_row(row: &AnyRow) -> Result<TimesheetRow, crate::Error> {
        Ok(TimesheetRow::new(
            TimesheetId::from_str(&row.try_get::<String, _>("id")?)?,
            UserId::from_str(&row.try_get::<String, _>("user_id")?)?,
            row.try_get::<String, _>("activity_id")
                .ok()
                .map(|activity_id| ActivityId::from_str(&activity_id))
                .transpose()?,
            row.try_get("start_time")?,
            row.try_get("end_time")?,
            row.try_get("duration")?,
            row.try_get("description")?,
            row.try_get("timezone")?,
        ))
    }
}

#[async_trait]
impl Getter<Timesheet> for TimesheetRepository {
    async fn get(&self, id: &TimesheetId) -> Result<Root<Timesheet>, GetError> {
        self.store.get(id).await
    }
}

#[async_trait]
impl Saver<Timesheet> for TimesheetRepository {
    async fn save(&self, root: &mut Root<Timesheet>) -> Result<(), SaveError> {
        self.store.save(root).await
    }
}

impl TimesheetRepositoryTrait for TimesheetRepository {}
