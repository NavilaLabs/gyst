use std::{ops::Deref, str::FromStr};

use async_trait::async_trait;
use eventually::aggregate::Root;
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use loom_core::admin::workspace::{
    Workspace, WorkspaceEvent, WorkspaceId, WorkspaceRepository as WorkspaceRepositoryTrait,
    WorkspaceView,
};
use loom_infrastructure::repository::{EntryToRow, ReadRepository};
use sea_query::{Alias, Condition, Expr, ExprTrait};
use sqlx::{Row, any::AnyRow, types::Uuid};

use crate::{
    ConnectedAdminPool, infrastructure::read_model::SeaQueryReadModel, snapshot::SnapshotRepository,
};

const TABLE: &str = "projections__workspaces";

pub struct WorkspaceRepository {
    store: SnapshotRepository<Workspace, ConnectedAdminPool>,
}

impl Deref for WorkspaceRepository {
    type Target = Repository<Workspace, Json<Workspace>, Json<WorkspaceEvent>>;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl WorkspaceRepository {
    /// # Errors
    ///
    /// Returns an error if the event store repository cannot be initialized.
    pub async fn from_pool(pool: ConnectedAdminPool) -> Result<Self, sqlx::migrate::MigrateError> {
        Ok(Self {
            store: SnapshotRepository::from_pool(pool).await?,
        })
    }

    #[must_use]
    pub const fn event_store(
        &self,
    ) -> &Repository<Workspace, Json<Workspace>, Json<WorkspaceEvent>> {
        self.store.event_store()
    }

    const fn read_model(&self) -> SeaQueryReadModel<'_> {
        SeaQueryReadModel::new(&self.store.pool, TABLE)
    }

    /// Returns all (`workspace_id`, `workspace_name`) pairs the given user belongs to.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_workspaces_for_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<(String, Option<String>)>, crate::Error> {
        let rows = sqlx::query(
            "SELECT DISTINCT w.id, w.name \
             FROM projections__workspaces w \
             INNER JOIN projections__workspace_user_roles r ON w.id = r.workspace_id \
             WHERE r.user_id = ?",
        )
        .bind(user_id)
        .fetch_all(self.store.pool.as_ref())
        .await?;

        rows.into_iter()
            .map(|row| -> Result<_, crate::Error> {
                Ok((
                    row.try_get::<String, _>("id")?,
                    row.try_get::<Option<String>, _>("name")?,
                ))
            })
            .collect()
    }

    /// Returns the first workspace ID the given user belongs to, or `None`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_workspace_for_user(
        &self,
        user_id: &str,
    ) -> Result<Option<String>, crate::Error> {
        let statement = sea_query::Query::select()
            .expr(Expr::col(Alias::new("workspace_id")))
            .from(Alias::new("projections__workspace_user_roles"))
            .and_where(Expr::col(Alias::new("user_id")).eq(user_id))
            .limit(1)
            .to_owned();

        let (sql, arguments) = self.store.pool.build_query(&statement);
        let row = sqlx::query_with(&sql, arguments)
            .fetch_optional(self.store.pool.as_ref())
            .await?;

        row.map(|r| r.try_get::<String, _>(0usize).map_err(crate::Error::from))
            .transpose()
    }

    /// Fetch a `WorkspaceView` by string ID, avoiding the `AnyPool` UUID-type panic.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_view_by_id(&self, id: &str) -> Result<Option<WorkspaceView>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm
            .select()
            .and_where(Expr::col(Alias::new("id")).eq(id))
            .to_owned();
        let row = rm.fetch_optional_row(&stmt).await?;
        row.map(|r| self.entry_to_row(r)).transpose()
    }
}

impl EntryToRow<AnyRow> for WorkspaceRepository {
    type Row = WorkspaceView;
    type Error = crate::Error;

    fn entry_to_row(&self, row: AnyRow) -> Result<WorkspaceView, crate::Error> {
        let id: String = row.try_get("id")?;
        let id = Uuid::from_str(&id)?;
        let name: Option<String> = row.try_get("name")?;
        let timezone: String = row
            .try_get("timezone")
            .unwrap_or_else(|_| "Europe/Berlin".to_string());
        let date_format: String = row
            .try_get("date_format")
            .unwrap_or_else(|_| "%Y-%m-%d".to_string());
        let currency: String = row
            .try_get("currency")
            .unwrap_or_else(|_| "EUR".to_string());
        let week_start: String = row
            .try_get("week_start")
            .unwrap_or_else(|_| "monday".to_string());
        Ok(WorkspaceView::new_with_settings(
            id.into(),
            name,
            timezone,
            date_format,
            currency,
            week_start,
        ))
    }
}

#[async_trait]
impl ReadRepository<AnyRow> for WorkspaceRepository {
    type Filter = Condition;

    async fn get_one(&self, id: Uuid) -> Result<WorkspaceView, crate::Error> {
        self.get_one_by(Condition::all().add(Expr::col("id").eq(id)))
            .await
    }

    async fn find_one(&self, id: Uuid) -> Result<Option<WorkspaceView>, crate::Error> {
        self.find_by(Condition::all().add(Expr::col("id").eq(id)))
            .await
    }

    async fn get_one_by(&self, filter: Condition) -> Result<WorkspaceView, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let row = rm.fetch_one_row(&stmt).await?;
        self.entry_to_row(row)
    }

    async fn find_by(&self, filter: Condition) -> Result<Option<WorkspaceView>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let row = rm.fetch_optional_row(&stmt).await?;
        row.map(|r| self.entry_to_row(r)).transpose()
    }

    async fn find_many(&self, ids: Vec<Uuid>) -> Result<Vec<WorkspaceView>, crate::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        self.find_many_by(Condition::all().add(Expr::col("id").is_in(ids)))
            .await
    }

    async fn find_many_by(&self, filter: Condition) -> Result<Vec<WorkspaceView>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let rows = rm.fetch_all_rows(&stmt).await?;
        rows.into_iter().map(|row| self.entry_to_row(row)).collect()
    }

    async fn all(&self) -> Result<Vec<WorkspaceView>, crate::Error> {
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
impl Getter<Workspace> for WorkspaceRepository {
    async fn get(&self, id: &WorkspaceId) -> Result<Root<Workspace>, GetError> {
        self.store.get(id).await
    }
}

#[async_trait]
impl Saver<Workspace> for WorkspaceRepository {
    async fn save(&self, root: &mut Root<Workspace>) -> Result<(), SaveError> {
        self.store.save(root).await
    }
}

impl WorkspaceRepositoryTrait for WorkspaceRepository {}
