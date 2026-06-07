use std::{collections::HashMap, ops::Deref, str::FromStr};

use async_trait::async_trait;
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::aggregate::{Aggregate, Root};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use sea_query::{Alias, Condition, Expr, ExprTrait, JoinType, Order};
use sqlx::{AssertSqlSafe, Row, any::AnyRow};
use zeitrak_core::shared::repositories::{ReadRepository, RowToRoot, WriteRepository};
use zeitrak_core::tenant::timesheet_tag::{
    TimesheetTag, TimesheetTagEvent, TimesheetTagId,
    TimesheetTagRepository as TimesheetTagRepositoryTrait, TimesheetTagRow,
};

use crate::{
    ConnectedTenantPool,
    infrastructure::{event_stream::current_stream_version, read_model::SeaQueryReadModel},
    snapshot::SnapshotRepository,
};

const TABLE: &str = "projections__timesheet_tags";
const JOIN_TABLE: &str = "projections__timesheet_has_tags";

pub struct TimesheetTagRepository {
    store: SnapshotRepository<TimesheetTag, ConnectedTenantPool>,
}

impl std::fmt::Debug for TimesheetTagRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimesheetTagRepository")
            .finish_non_exhaustive()
    }
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

    const fn read_model(&self) -> SeaQueryReadModel<'_, ConnectedTenantPool> {
        SeaQueryReadModel::new(&self.store.pool, TABLE)
    }

    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn all(&self) -> Result<Vec<TimesheetTagRow>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm
            .select()
            .order_by(Alias::new("name"), Order::Asc)
            .to_owned();
        let rows = rm.fetch_all_rows(&stmt).await?;
        rows.into_iter().map(|r| Self::map_row(&r)).collect()
    }

    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn for_timesheet(
        &self,
        timesheet_id: &str,
    ) -> Result<Vec<TimesheetTagRow>, crate::Error> {
        let stmt = sea_query::Query::select()
            .column((Alias::new("t"), Alias::new("id")))
            .column((Alias::new("t"), Alias::new("name")))
            .from_as(Alias::new(TABLE), Alias::new("t"))
            .join_as(
                JoinType::InnerJoin,
                Alias::new(JOIN_TABLE),
                Alias::new("tht"),
                Expr::col((Alias::new("tht"), Alias::new("timesheet_tag_id")))
                    .equals((Alias::new("t"), Alias::new("id"))),
            )
            .and_where(Expr::col((Alias::new("tht"), Alias::new("timesheet_id"))).eq(timesheet_id))
            .order_by((Alias::new("t"), Alias::new("name")), Order::Asc)
            .to_owned();
        let (sql, values) = self.store.pool.build_query(&stmt);
        let rows = sqlx::query_with(AssertSqlSafe(sql.as_str()), values)
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
        let ids: Vec<String> = timesheet_ids.iter().map(|s| (*s).to_string()).collect();
        let stmt = sea_query::Query::select()
            .column((Alias::new("tht"), Alias::new("timesheet_id")))
            .column((Alias::new("t"), Alias::new("id")))
            .column((Alias::new("t"), Alias::new("name")))
            .from_as(Alias::new(JOIN_TABLE), Alias::new("tht"))
            .join_as(
                JoinType::InnerJoin,
                Alias::new(TABLE),
                Alias::new("t"),
                Expr::col((Alias::new("t"), Alias::new("id")))
                    .equals((Alias::new("tht"), Alias::new("timesheet_tag_id"))),
            )
            .and_where(Expr::col((Alias::new("tht"), Alias::new("timesheet_id"))).is_in(ids))
            .order_by((Alias::new("t"), Alias::new("name")), Order::Asc)
            .to_owned();
        let (sql, values) = self.store.pool.build_query(&stmt);
        let rows = sqlx::query_with(AssertSqlSafe(sql.as_str()), values)
            .fetch_all(self.store.pool.as_ref())
            .await?;
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

impl RowToRoot<AnyRow, TimesheetTag> for TimesheetTagRepository {
    type Error = crate::Error;

    fn row_to_root(&self, row: AnyRow) -> Result<Root<TimesheetTag>, crate::Error> {
        let id: String = row.try_get("id")?;
        let id = TimesheetTagId::from_str(&id)?;
        let name: String = row.try_get("name")?;
        let tag = TimesheetTag::apply(None, TimesheetTagEvent::Created { id, name })
            .expect("Created event on None state is infallible");
        Ok(Root::rehydrate_from_state(0, tag))
    }
}

impl TimesheetTagRepository {
    async fn row_to_root_versioned(&self, row: AnyRow) -> Result<Root<TimesheetTag>, crate::Error> {
        let root = self.row_to_root(row)?;
        let version =
            current_stream_version(&self.store.pool, &root.aggregate_id().to_string()).await?;
        Ok(Root::rehydrate_from_state(
            version,
            root.to_aggregate_type::<TimesheetTag>(),
        ))
    }
}

impl zeitrak_core::shared::repositories::Repository<TimesheetTag, AnyRow>
    for TimesheetTagRepository
{
}

#[async_trait]
impl ReadRepository<TimesheetTag, AnyRow> for TimesheetTagRepository {
    type Error = crate::Error;
    type Filter = Condition;

    async fn find(&self, id: TimesheetTagId) -> Result<Option<Root<TimesheetTag>>, crate::Error> {
        self.find_by(Condition::all().add(Expr::col("id").eq(id.to_string())))
            .await
    }

    async fn find_by(&self, filter: Condition) -> Result<Option<Root<TimesheetTag>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let row = rm.fetch_optional_row(&stmt).await?;
        if let Some(row) = row {
            Ok(Some(self.row_to_root_versioned(row).await?))
        } else {
            Ok(None)
        }
    }

    async fn find_many(
        &self,
        ids: Vec<TimesheetTagId>,
    ) -> Result<Vec<Root<TimesheetTag>>, crate::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let id_strings: Vec<String> = ids.into_iter().map(|id| id.to_string()).collect();
        self.find_many_by(Condition::all().add(Expr::col("id").is_in(id_strings)))
            .await
    }

    async fn find_many_by(
        &self,
        filter: Condition,
    ) -> Result<Vec<Root<TimesheetTag>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let rows = rm.fetch_all_rows(&stmt).await?;
        let mut roots = Vec::with_capacity(rows.len());
        for row in rows {
            roots.push(self.row_to_root_versioned(row).await?);
        }
        Ok(roots)
    }

    async fn all(&self) -> Result<Vec<Root<TimesheetTag>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select();
        let rows = rm.fetch_all_rows(&stmt).await?;
        let mut roots = Vec::with_capacity(rows.len());
        for row in rows {
            roots.push(self.row_to_root_versioned(row).await?);
        }
        Ok(roots)
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
impl WriteRepository<TimesheetTag> for TimesheetTagRepository {
    type Error = crate::Error;
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

#[async_trait]
impl TimesheetTagRepositoryTrait<AnyRow> for TimesheetTagRepository {
    type Error = crate::Error;

    async fn list_all(&self) -> Result<Vec<TimesheetTagRow>, crate::Error> {
        self.all().await
    }

    async fn for_timesheet(
        &self,
        timesheet_id: &str,
    ) -> Result<Vec<TimesheetTagRow>, crate::Error> {
        self.for_timesheet(timesheet_id).await
    }

    async fn for_timesheets_batch(
        &self,
        timesheet_ids: &[&str],
    ) -> Result<HashMap<String, Vec<TimesheetTagRow>>, crate::Error> {
        self.for_timesheets_batch(timesheet_ids).await
    }
}
