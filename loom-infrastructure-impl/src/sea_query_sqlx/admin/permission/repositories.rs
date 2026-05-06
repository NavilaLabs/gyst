use std::{ops::Deref, str::FromStr};

use async_trait::async_trait;
use eventually::aggregate::{Aggregate, Root};
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use loom_core::admin::permission::{Permission, PermissionEvent, PermissionId, PermissionRepository as PermissionRepositoryTrait};
use loom_core::shared::repositories::{ReadRepository, WriteRepository};
use sea_query::{Condition, Expr, ExprTrait};
use sqlx::{Row, any::AnyRow};

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

    fn entry_to_row(&self, row: AnyRow) -> Result<Root<Permission>, crate::Error> {
        let id: String = row.try_get("id")?;
        let id = PermissionId::from_str(&id)?;
        let name: String = row.try_get("name")?;
        let permission = Permission::apply(None, PermissionEvent::Created { id, name })
            .expect("Created event on None state is infallible");
        Ok(Root::rehydrate_from_state(0, permission))
    }
}

#[async_trait]
impl Getter<Permission> for PermissionRepository {
    async fn get(&self, id: &PermissionId) -> Result<Root<Permission>, GetError> {
        self.store.get(id).await
    }
}

#[async_trait]
impl Saver<Permission> for PermissionRepository {
    async fn save(&self, root: &mut Root<Permission>) -> Result<(), SaveError> {
        self.store.save(root).await?;
        Ok(())
    }
}

#[async_trait]
impl ReadRepository<Permission> for PermissionRepository {
    type Error = crate::Error;
    type Filter = Condition;

    async fn find(&self, id: PermissionId) -> Result<Option<Root<Permission>>, Self::Error> {
        self.find_by(Condition::all().add(Expr::col("id").eq(id.to_string())))
            .await
    }

    async fn find_by(&self, filter: Condition) -> Result<Option<Root<Permission>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let row = rm.fetch_optional_row(&stmt).await?;
        row.map(|r| self.entry_to_row(r)).transpose()
    }

    async fn find_many(&self, ids: Vec<PermissionId>) -> Result<Vec<Root<Permission>>, crate::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let id_strings: Vec<String> = ids.into_iter().map(|id| id.to_string()).collect();
        self.find_many_by(Condition::all().add(Expr::col("id").is_in(id_strings)))
            .await
    }

    async fn find_many_by(&self, filter: Condition) -> Result<Vec<Root<Permission>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let rows = rm.fetch_all_rows(&stmt).await?;
        rows.into_iter().map(|row| self.entry_to_row(row)).collect()
    }

    async fn all(&self) -> Result<Vec<Root<Permission>>, crate::Error> {
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
impl WriteRepository<Permission> for PermissionRepository {
    type Error = crate::Error;
}

#[async_trait]
impl PermissionRepositoryTrait for PermissionRepository {
    type Error = crate::Error;
}
