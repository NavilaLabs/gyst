use std::{ops::Deref, str::FromStr};

use async_trait::async_trait;
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use loom_core::admin::workspace_role::{
    WorkspaceRole, WorkspaceRoleEvent, WorkspaceRoleId, WorkspaceRoleView,
};
use loom_infrastructure::query::{Query, RowToView};
use sea_query::{Condition, Expr, ExprTrait, Func, SelectStatement};
use sqlx::{Row, any::AnyRow, types::Uuid};

use crate::ConnectedAdminPool;

const TABLE: &str = "projections__workspace_roles";

pub struct WorkspaceRoleRepository {
    database: ConnectedAdminPool,
    repository: Repository<WorkspaceRole, Json<WorkspaceRole>, Json<WorkspaceRoleEvent>>,
}

impl Deref for WorkspaceRoleRepository {
    type Target = Repository<WorkspaceRole, Json<WorkspaceRole>, Json<WorkspaceRoleEvent>>;

    fn deref(&self) -> &Self::Target {
        &self.repository
    }
}

impl WorkspaceRoleRepository {
    #[must_use]
    pub const fn new(
        database: ConnectedAdminPool,
        repository: Repository<WorkspaceRole, Json<WorkspaceRole>, Json<WorkspaceRoleEvent>>,
    ) -> Self {
        Self {
            database,
            repository,
        }
    }

    /// # Errors
    ///
    /// Returns an error if the event store repository cannot be initialized.
    pub async fn from_pool(pool: ConnectedAdminPool) -> Result<Self, sqlx::migrate::MigrateError> {
        let repository =
            Repository::new(pool.as_ref().clone(), Json::default(), Json::default()).await?;
        Ok(Self {
            database: pool,
            repository,
        })
    }

    #[must_use]
    pub const fn event_store(
        &self,
    ) -> &Repository<WorkspaceRole, Json<WorkspaceRole>, Json<WorkspaceRoleEvent>> {
        &self.repository
    }

    #[allow(clippy::unused_self)]
    fn select(&self) -> SelectStatement {
        sea_query::Query::select()
            .expr(Expr::col(sea_query::Asterisk))
            .from(TABLE)
            .to_owned()
    }

    #[allow(clippy::unused_self)]
    fn select_count(&self) -> SelectStatement {
        sea_query::Query::select()
            .expr(Func::count(Expr::col(sea_query::Asterisk)))
            .from(TABLE)
            .to_owned()
    }
}

impl RowToView<AnyRow> for WorkspaceRoleRepository {
    type View = WorkspaceRoleView;
    type Error = crate::Error;

    fn row_to_view(&self, row: AnyRow) -> Result<WorkspaceRoleView, crate::Error> {
        let id: String = row.try_get("id")?;
        let id = Uuid::from_str(&id)?;
        let workspace_id: String = row.try_get("workspace_id")?;
        let workspace_id = Uuid::from_str(&workspace_id)?;
        let name: Option<String> = row.try_get("name")?;
        Ok(WorkspaceRoleView::new(id.into(), workspace_id.into(), name))
    }
}

#[async_trait]
impl Query<AnyRow> for WorkspaceRoleRepository {
    type Filter = Condition;

    async fn get_one(&self, id: Uuid) -> Result<WorkspaceRoleView, crate::Error> {
        self.get_one_by(Condition::all().add(Expr::col("id").eq(id.to_string())))
            .await
    }

    async fn find_one(&self, id: Uuid) -> Result<Option<WorkspaceRoleView>, crate::Error> {
        self.find_one_by(Condition::all().add(Expr::col("id").eq(id.to_string())))
            .await
    }

    async fn get_one_by(&self, filter: Condition) -> Result<WorkspaceRoleView, crate::Error> {
        let statement = self.select().cond_where(filter).to_owned();
        let (sql, arguments) = self.database.build_query(&statement);
        let row = sqlx::query_with(&sql, arguments)
            .fetch_one(self.database.as_ref())
            .await?;
        self.row_to_view(row)
    }

    async fn find_one_by(
        &self,
        filter: Condition,
    ) -> Result<Option<WorkspaceRoleView>, crate::Error> {
        let statement = self.select().cond_where(filter).to_owned();
        let (sql, arguments) = self.database.build_query(&statement);
        let row = sqlx::query_with(&sql, arguments)
            .fetch_optional(self.database.as_ref())
            .await?;
        row.map(|r| self.row_to_view(r)).transpose()
    }

    async fn find_many(&self, ids: Vec<Uuid>) -> Result<Vec<WorkspaceRoleView>, crate::Error> {
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
    ) -> Result<Vec<WorkspaceRoleView>, crate::Error> {
        let statement = self.select().cond_where(filter).to_owned();
        let (sql, arguments) = self.database.build_query(&statement);
        let rows = sqlx::query_with(&sql, arguments)
            .fetch_all(self.database.as_ref())
            .await?;
        rows.into_iter().map(|row| self.row_to_view(row)).collect()
    }

    async fn all(&self) -> Result<Vec<WorkspaceRoleView>, crate::Error> {
        let (sql, arguments) = self.database.build_query(&self.select());
        let rows = sqlx::query_with(&sql, arguments)
            .fetch_all(self.database.as_ref())
            .await?;
        rows.into_iter().map(|row| self.row_to_view(row)).collect()
    }

    async fn count_by(&self, filter: Condition) -> Result<u64, crate::Error> {
        let statement = self.select_count().cond_where(filter).to_owned();
        let (sql, arguments) = self.database.build_query(&statement);
        let row = sqlx::query_with(&sql, arguments)
            .fetch_one(self.database.as_ref())
            .await?;
        let n: i64 = row.try_get(0)?;
        #[allow(clippy::cast_sign_loss)]
        Ok(n as u64)
    }

    async fn count(&self) -> Result<u64, crate::Error> {
        let (sql, arguments) = self.database.build_query(&self.select_count());
        let row = sqlx::query_with(&sql, arguments)
            .fetch_one(self.database.as_ref())
            .await?;
        let n: i64 = row.try_get(0)?;
        #[allow(clippy::cast_sign_loss)]
        Ok(n as u64)
    }
}

#[async_trait]
impl Getter<WorkspaceRole> for WorkspaceRoleRepository {
    async fn get(
        &self,
        id: &WorkspaceRoleId,
    ) -> Result<eventually::aggregate::Root<WorkspaceRole>, GetError> {
        self.repository.get(id).await
    }
}

#[async_trait]
impl Saver<WorkspaceRole> for WorkspaceRoleRepository {
    async fn save(
        &self,
        root: &mut eventually::aggregate::Root<WorkspaceRole>,
    ) -> Result<(), SaveError> {
        self.repository.save(root).await
    }
}
