use std::{ops::Deref, str::FromStr};

use async_trait::async_trait;
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::aggregate::{Aggregate, Root};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use sea_query::{Alias, Condition, Expr, ExprTrait, JoinType, Order};
use sqlx::{Row, any::AnyRow};
use zeitrak_core::admin::workspace::{
    Workspace, WorkspaceEvent, WorkspaceId, WorkspaceRepository as WorkspaceRepositoryTrait,
    WorkspaceRow,
};
use zeitrak_core::shared::repositories::{ReadRepository, RowToRoot, WriteRepository};

use crate::{
    ConnectedAdminPool, infrastructure::read_model::SeaQueryReadModel, snapshot::SnapshotRepository,
};

const TABLE: &str = "projections__workspaces";

pub struct WorkspaceRepository {
    store: SnapshotRepository<Workspace, ConnectedAdminPool>,
}

impl std::fmt::Debug for WorkspaceRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkspaceRepository")
            .finish_non_exhaustive()
    }
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

    #[allow(clippy::unused_self)]
    fn row_to_view(&self, row: &AnyRow) -> Result<WorkspaceRow, crate::Error> {
        let id: String = row.try_get("id")?;
        let id = WorkspaceId::from_str(&id)?;
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
        Ok(WorkspaceRow::new_with_settings(
            id,
            name,
            timezone,
            date_format,
            currency,
            week_start,
        ))
    }

    async fn find_workspaces_for_user_impl(
        &self,
        user_id: &str,
    ) -> Result<Vec<(String, Option<String>)>, crate::Error> {
        const ROLES_TABLE: &str = "projections__workspace_user_roles";
        let stmt = sea_query::Query::select()
            .distinct()
            .column((Alias::new("w"), Alias::new("id")))
            .column((Alias::new("w"), Alias::new("name")))
            .from_as(Alias::new(TABLE), Alias::new("w"))
            .join_as(
                JoinType::InnerJoin,
                Alias::new(ROLES_TABLE),
                Alias::new("r"),
                Expr::col((Alias::new("w"), Alias::new("id")))
                    .equals((Alias::new("r"), Alias::new("workspace_id"))),
            )
            .and_where(Expr::col((Alias::new("r"), Alias::new("user_id"))).eq(user_id))
            .order_by((Alias::new("w"), Alias::new("id")), Order::Asc)
            .to_owned();
        let (sql, values) = self.store.pool.build_query(&stmt);
        let rows = sqlx::query_with(&sql, values)
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

    async fn find_workspace_for_user_impl(
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

    async fn find_view_by_id_impl(&self, id: &str) -> Result<Option<WorkspaceRow>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm
            .select()
            .and_where(Expr::col(Alias::new("id")).eq(id))
            .to_owned();
        let row = rm.fetch_optional_row(&stmt).await?;
        row.map(|r| self.row_to_view(&r)).transpose()
    }
}

impl RowToRoot<AnyRow, Workspace> for WorkspaceRepository {
    type Error = crate::Error;

    fn row_to_root(&self, row: AnyRow) -> Result<Root<Workspace>, crate::Error> {
        let id: String = row.try_get("id")?;
        let id = WorkspaceId::from_str(&id)?;
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
        let workspace = Workspace::apply(
            None,
            WorkspaceEvent::Created {
                id,
                name: name.clone(),
            },
        )
        .expect("Created event on None state is infallible");
        let workspace = Workspace::apply(
            Some(workspace),
            WorkspaceEvent::SettingsUpdated {
                name,
                timezone,
                date_format,
                currency,
                week_start,
            },
        )
        .expect("SettingsUpdated event on Some state is infallible");
        Ok(Root::rehydrate_from_state(0, workspace))
    }
}

impl zeitrak_core::shared::repositories::Repository<Workspace, AnyRow> for WorkspaceRepository {}

#[async_trait]
impl ReadRepository<Workspace, AnyRow> for WorkspaceRepository {
    type Error = crate::Error;
    type Filter = Condition;

    async fn find(&self, id: WorkspaceId) -> Result<Option<Root<Workspace>>, crate::Error> {
        self.find_by(Condition::all().add(Expr::col("id").eq(id.to_string())))
            .await
    }

    async fn find_by(&self, filter: Condition) -> Result<Option<Root<Workspace>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let row = rm.fetch_optional_row(&stmt).await?;
        row.map(|r| self.row_to_root(r)).transpose()
    }

    async fn find_many(&self, ids: Vec<WorkspaceId>) -> Result<Vec<Root<Workspace>>, crate::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let id_strings: Vec<String> = ids.into_iter().map(|id| id.to_string()).collect();
        self.find_many_by(Condition::all().add(Expr::col("id").is_in(id_strings)))
            .await
    }

    async fn find_many_by(&self, filter: Condition) -> Result<Vec<Root<Workspace>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let rows = rm.fetch_all_rows(&stmt).await?;
        rows.into_iter().map(|row| self.row_to_root(row)).collect()
    }

    async fn all(&self) -> Result<Vec<Root<Workspace>>, crate::Error> {
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
impl WriteRepository<Workspace> for WorkspaceRepository {
    type Error = crate::Error;
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

#[async_trait]
impl WorkspaceRepositoryTrait<AnyRow> for WorkspaceRepository {
    type Error = crate::Error;

    async fn find_workspaces_for_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<(String, Option<String>)>, crate::Error> {
        self.find_workspaces_for_user_impl(user_id).await
    }

    async fn find_workspace_for_user(&self, user_id: &str) -> Result<Option<String>, crate::Error> {
        self.find_workspace_for_user_impl(user_id).await
    }

    async fn find_view_by_id(&self, id: &str) -> Result<Option<WorkspaceRow>, crate::Error> {
        self.find_view_by_id_impl(id).await
    }
}
