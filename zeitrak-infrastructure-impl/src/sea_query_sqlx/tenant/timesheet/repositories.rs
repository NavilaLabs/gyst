use std::{ops::Deref, str::FromStr};

use async_trait::async_trait;
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::aggregate::{Aggregate, Root};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use sea_query::{Condition, Expr, ExprTrait};
use sqlx::{Row, any::AnyRow};
use zeitrak_core::admin::user::UserId;
use zeitrak_core::shared::repositories::{ReadRepository, RowToRoot, WriteRepository};
use zeitrak_core::tenant::activity::ActivityId;
use zeitrak_core::tenant::timesheet::{
    Timesheet, TimesheetEvent, TimesheetId, TimesheetRepository as TimesheetRepositoryTrait,
    TimesheetRow,
};

use crate::{
    ConnectedTenantPool, infrastructure::read_model::SeaQueryReadModel,
    snapshot::SnapshotRepository,
};

const TABLE: &str = "projections__timesheets";

pub struct TimesheetRepository {
    store: SnapshotRepository<Timesheet, ConnectedTenantPool>,
}

impl std::fmt::Debug for TimesheetRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimesheetRepository")
            .finish_non_exhaustive()
    }
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

    const fn read_model(&self) -> SeaQueryReadModel<'_, ConnectedTenantPool> {
        SeaQueryReadModel::new(&self.store.pool, TABLE)
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

impl RowToRoot<AnyRow, Timesheet> for TimesheetRepository {
    type Error = crate::Error;

    fn row_to_root(&self, row: AnyRow) -> Result<Root<Timesheet>, crate::Error> {
        let id: String = row.try_get("id")?;
        let id = TimesheetId::from_str(&id)?;
        let user_id: String = row.try_get("user_id")?;
        let user_id = UserId::from_str(&user_id)?;
        let activity_id = row
            .try_get::<String, _>("activity_id")
            .ok()
            .map(|s| ActivityId::from_str(&s))
            .transpose()?;
        let start_time: String = row.try_get("start_time")?;
        let timezone: String = row.try_get("timezone")?;
        let end_time: Option<String> = row.try_get("end_time")?;
        let duration: Option<i32> = row.try_get("duration")?;
        let description: Option<String> = row.try_get("description")?;
        let cancelled_at: Option<String> = row.try_get("cancelled_at").ok().flatten();

        let ts = Timesheet::apply(
            None,
            TimesheetEvent::Started {
                id,
                user_id,
                activity_id,
                start_time,
                timezone,
            },
        )
        .expect("Started event on None state is infallible");

        let ts = match (end_time, duration) {
            (Some(end_time), Some(duration)) => {
                Timesheet::apply(Some(ts), TimesheetEvent::Stopped { end_time, duration })
                    .expect("Stopped event on Some state is infallible")
            }
            _ => ts,
        };

        let ts = match description {
            Some(desc) => Timesheet::apply(
                Some(ts),
                TimesheetEvent::Updated {
                    description: Some(desc),
                },
            )
            .expect("Updated event on Some state is infallible"),
            None => ts,
        };

        let ts = if cancelled_at.is_some() {
            Timesheet::apply(Some(ts), TimesheetEvent::Cancelled {})
                .expect("Cancelled event on Some state is infallible")
        } else {
            ts
        };

        Ok(Root::rehydrate_from_state(0, ts))
    }
}

impl zeitrak_core::shared::repositories::Repository<Timesheet, AnyRow> for TimesheetRepository {}

#[async_trait]
impl ReadRepository<Timesheet, AnyRow> for TimesheetRepository {
    type Error = crate::Error;
    type Filter = Condition;

    async fn find(&self, id: TimesheetId) -> Result<Option<Root<Timesheet>>, crate::Error> {
        self.find_by(Condition::all().add(Expr::col("id").eq(id.to_string())))
            .await
    }

    async fn find_by(&self, filter: Condition) -> Result<Option<Root<Timesheet>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let row = rm.fetch_optional_row(&stmt).await?;
        row.map(|r| self.row_to_root(r)).transpose()
    }

    async fn find_many(&self, ids: Vec<TimesheetId>) -> Result<Vec<Root<Timesheet>>, crate::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let id_strings: Vec<String> = ids.into_iter().map(|id| id.to_string()).collect();
        self.find_many_by(Condition::all().add(Expr::col("id").is_in(id_strings)))
            .await
    }

    async fn find_many_by(&self, filter: Condition) -> Result<Vec<Root<Timesheet>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let rows = rm.fetch_all_rows(&stmt).await?;
        rows.into_iter().map(|row| self.row_to_root(row)).collect()
    }

    async fn all(&self) -> Result<Vec<Root<Timesheet>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select();
        let rows = rm.fetch_all_rows(&stmt).await?;
        rows.into_iter().map(|row| self.row_to_root(row)).collect()
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
impl WriteRepository<Timesheet> for TimesheetRepository {
    type Error = crate::Error;
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

#[async_trait]
impl TimesheetRepositoryTrait<AnyRow> for TimesheetRepository {
    type Error = crate::Error;

    async fn recent_for_user(&self, user_id: &str) -> Result<Vec<TimesheetRow>, crate::Error> {
        self.recent_for_user(user_id).await
    }

    async fn running_for_user(&self, user_id: &str) -> Result<Option<TimesheetRow>, crate::Error> {
        self.running_for_user(user_id).await
    }
}
