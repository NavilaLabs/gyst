use std::{ops::Deref, str::FromStr};

use async_trait::async_trait;
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use loom_core::admin::permission::{Permission, PermissionEvent, PermissionView};
use loom_infrastructure::query::{Query, RowToView};
use sea_query::{Condition, Expr, ExprTrait};
use sqlx::{Row, any::AnyRow, types::Uuid};

use crate::{
    ConnectedAdminPool, infrastructure::read_model::SeaQueryReadModel, snapshot::SnapshotRepository,
};

const TABLE: &str = "permissions";

pub struct PermissionRepository {
    store: SnapshotRepository<Permission, ConnectedAdminPool>,
}

impl Deref for PermissionRepository {
    type Target = Repository<Permission, Json<Permission>, Json<PermissionEvent>>;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl PermissionRepository {
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
    ) -> &Repository<Permission, Json<Permission>, Json<PermissionEvent>> {
        self.store.event_store()
    }

    const fn read_model(&self) -> SeaQueryReadModel<'_> {
        SeaQueryReadModel::new(&self.store.pool, TABLE)
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
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let row = rm.fetch_one_row(&stmt).await?;
        self.row_to_view(row)
    }

    async fn find_one_by(&self, filter: Condition) -> Result<Option<PermissionView>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let row = rm.fetch_optional_row(&stmt).await?;
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
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let rows = rm.fetch_all_rows(&stmt).await?;
        rows.into_iter().map(|row| self.row_to_view(row)).collect()
    }

    async fn all(&self) -> Result<Vec<PermissionView>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select();
        let rows = rm.fetch_all_rows(&stmt).await?;
        rows.into_iter().map(|row| self.row_to_view(row)).collect()
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
