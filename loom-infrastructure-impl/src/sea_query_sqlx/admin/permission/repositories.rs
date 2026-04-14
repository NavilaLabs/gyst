use std::{ops::Deref, str::FromStr};

use async_trait::async_trait;
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use loom_core::admin::permission::{Permission, PermissionEvent, PermissionView};
use loom_infrastructure::query::{Query, RowToView};
use sea_query::{Condition, Expr, ExprTrait, Func, SelectStatement};
use sqlx::{Row, any::AnyRow, types::Uuid};

use crate::ConnectedAdminPool;

const TABLE: &str = "permissions";

pub struct PermissionRepository {
    database: ConnectedAdminPool,
    repository: Repository<Permission, Json<Permission>, Json<PermissionEvent>>,
}

impl Deref for PermissionRepository {
    type Target = Repository<Permission, Json<Permission>, Json<PermissionEvent>>;

    fn deref(&self) -> &Self::Target {
        &self.repository
    }
}

impl PermissionRepository {
    #[must_use]
    pub const fn new(
        database: ConnectedAdminPool,
        repository: Repository<Permission, Json<Permission>, Json<PermissionEvent>>,
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
    ) -> &Repository<Permission, Json<Permission>, Json<PermissionEvent>> {
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

impl RowToView<AnyRow> for PermissionRepository {
    type View = PermissionView;
    type Error = crate::Error;

    fn row_to_view(&self, row: AnyRow) -> Result<PermissionView, crate::Error> {
        let id: String = row.try_get("id")?;
        let id = Uuid::from_str(&id)?;
        let name: String = row.try_get("name")?;
        Ok(PermissionView::new(id.into(), name))
    }
}

#[async_trait]
impl Query<AnyRow> for PermissionRepository {
    type Filter = Condition;

    async fn get_one(&self, id: Uuid) -> Result<PermissionView, crate::Error> {
        self.get_one_by(Condition::all().add(Expr::col("id").eq(id.to_string())))
            .await
    }

    async fn find_one(&self, id: Uuid) -> Result<Option<PermissionView>, crate::Error> {
        self.find_one_by(Condition::all().add(Expr::col("id").eq(id.to_string())))
            .await
    }

    async fn get_one_by(&self, filter: Condition) -> Result<PermissionView, crate::Error> {
        let statement = self.select().cond_where(filter).to_owned();
        let (sql, arguments) = self.database.build_query(&statement);
        let row = sqlx::query_with(&sql, arguments)
            .fetch_one(self.database.as_ref())
            .await?;
        self.row_to_view(row)
    }

    async fn find_one_by(&self, filter: Condition) -> Result<Option<PermissionView>, crate::Error> {
        let statement = self.select().cond_where(filter).to_owned();
        let (sql, arguments) = self.database.build_query(&statement);
        let row = sqlx::query_with(&sql, arguments)
            .fetch_optional(self.database.as_ref())
            .await?;
        row.map(|r| self.row_to_view(r)).transpose()
    }

    async fn find_many(&self, ids: Vec<Uuid>) -> Result<Vec<PermissionView>, crate::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let id_strings: Vec<String> = ids.into_iter().map(|id| id.to_string()).collect();
        self.find_many_by(Condition::all().add(Expr::col("id").is_in(id_strings)))
            .await
    }

    async fn find_many_by(&self, filter: Condition) -> Result<Vec<PermissionView>, crate::Error> {
        let statement = self.select().cond_where(filter).to_owned();
        let (sql, arguments) = self.database.build_query(&statement);
        let rows = sqlx::query_with(&sql, arguments)
            .fetch_all(self.database.as_ref())
            .await?;
        rows.into_iter().map(|row| self.row_to_view(row)).collect()
    }

    async fn all(&self) -> Result<Vec<PermissionView>, crate::Error> {
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
