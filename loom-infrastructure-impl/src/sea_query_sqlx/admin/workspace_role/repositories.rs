use std::{ops::Deref, str::FromStr};

use async_trait::async_trait;
use eventually::aggregate::Root;
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use loom_core::admin::workspace_role::{
    WorkspaceRole, WorkspaceRoleEvent, WorkspaceRoleId, WorkspaceRoleView,
};
use loom_infrastructure::repository::{ReadRepository, EntryToRow};
use sea_query::{Condition, Expr, ExprTrait};
use sqlx::{Row, any::AnyRow, types::Uuid};

use crate::{
    ConnectedAdminPool, infrastructure::read_model::SeaQueryReadModel, snapshot::SnapshotRepository,
};

const TABLE: &str = "projections__workspace_roles";

pub struct WorkspaceRoleRepository {
    store: SnapshotRepository<WorkspaceRole, ConnectedAdminPool>,
}

impl Deref for WorkspaceRoleRepository {
    type Target = Repository<WorkspaceRole, Json<WorkspaceRole>, Json<WorkspaceRoleEvent>>;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl WorkspaceRoleRepository {
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
    ) -> &Repository<WorkspaceRole, Json<WorkspaceRole>, Json<WorkspaceRoleEvent>> {
        self.store.event_store()
    }

    const fn read_model(&self) -> SeaQueryReadModel<'_> {
        SeaQueryReadModel::new(&self.store.pool, TABLE)
    }
}

impl EntryToRow<AnyRow> for WorkspaceRoleRepository {
    type Row = WorkspaceRoleView;
    type Error = crate::Error;

    fn entry_to_row(&self, row: AnyRow) -> Result<WorkspaceRoleView, crate::Error> {
        let id: String = row.try_get("id")?;
        let id = Uuid::from_str(&id)?;
        let workspace_id: String = row.try_get("workspace_id")?;
        let workspace_id = Uuid::from_str(&workspace_id)?;
        let name: Option<String> = row.try_get("name")?;
        Ok(WorkspaceRoleView::new(id.into(), workspace_id.into(), name))
    }
}

#[async_trait]
impl ReadRepository<AnyRow> for WorkspaceRoleRepository {
    type Filter = Condition;

    async fn get_one(&self, id: Uuid) -> Result<WorkspaceRoleView, crate::Error> {
        self.get_one_by(Condition::all().add(Expr::col("id").eq(id.to_string())))
            .await
    }

    async fn find_one(&self, id: Uuid) -> Result<Option<WorkspaceRoleView>, crate::Error> {
        self.find_by(Condition::all().add(Expr::col("id").eq(id.to_string())))
            .await
    }

    async fn get_one_by(&self, filter: Condition) -> Result<WorkspaceRoleView, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let row = rm.fetch_one_row(&stmt).await?;
        self.entry_to_row(row)
    }

    async fn find_by(
        &self,
        filter: Condition,
    ) -> Result<Option<WorkspaceRoleView>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let row = rm.fetch_optional_row(&stmt).await?;
        row.map(|r| self.entry_to_row(r)).transpose()
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
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let rows = rm.fetch_all_rows(&stmt).await?;
        rows.into_iter().map(|row| self.entry_to_row(row)).collect()
    }

    async fn all(&self) -> Result<Vec<WorkspaceRoleView>, crate::Error> {
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
impl Getter<WorkspaceRole> for WorkspaceRoleRepository {
    async fn get(&self, id: &WorkspaceRoleId) -> Result<Root<WorkspaceRole>, GetError> {
        self.store.get(id).await
    }
}

#[async_trait]
impl Saver<WorkspaceRole> for WorkspaceRoleRepository {
    async fn save(&self, root: &mut Root<WorkspaceRole>) -> Result<(), SaveError> {
        self.store.save(root).await
    }
}
