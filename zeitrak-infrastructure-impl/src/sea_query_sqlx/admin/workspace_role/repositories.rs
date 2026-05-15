use std::{ops::Deref, str::FromStr};

use async_trait::async_trait;
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::aggregate::{Aggregate, Root};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use sea_query::{Condition, Expr, ExprTrait};
use sqlx::{Row, any::AnyRow};
use zeitrak_core::admin::workspace::WorkspaceId;
use zeitrak_core::admin::workspace_role::{
    WorkspaceRole, WorkspaceRoleEvent, WorkspaceRoleId,
    WorkspaceRoleRepository as WorkspaceRoleRepositoryTrait,
    WorkspaceRoleWithPermissionsRow,
};
use zeitrak_core::shared::repositories::{ReadRepository, RowToRoot, WriteRepository};

use crate::{
    ConnectedAdminPool, infrastructure::read_model::SeaQueryReadModel, snapshot::SnapshotRepository,
};

const TABLE: &str = "projections__workspace_roles";

pub struct WorkspaceRoleRepository {
    store: SnapshotRepository<WorkspaceRole, ConnectedAdminPool>,
}

impl std::fmt::Debug for WorkspaceRoleRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkspaceRoleRepository")
            .finish_non_exhaustive()
    }
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

impl RowToRoot<AnyRow, WorkspaceRole> for WorkspaceRoleRepository {
    type Error = crate::Error;

    fn row_to_root(&self, row: AnyRow) -> Result<Root<WorkspaceRole>, crate::Error> {
        let id: String = row.try_get("id")?;
        let id = WorkspaceRoleId::from_str(&id)?;
        let workspace_id: String = row.try_get("workspace_id")?;
        let workspace_id = WorkspaceId::from_str(&workspace_id)?;
        let name: Option<String> = row.try_get("name")?;
        let role = WorkspaceRole::apply(
            None,
            WorkspaceRoleEvent::Created {
                id,
                workspace_id,
                name,
            },
        )
        .expect("Created event on None state is infallible");
        Ok(Root::rehydrate_from_state(0, role))
    }
}

impl zeitrak_core::shared::repositories::Repository<WorkspaceRole, AnyRow>
    for WorkspaceRoleRepository
{
}

#[async_trait]
impl ReadRepository<WorkspaceRole, AnyRow> for WorkspaceRoleRepository {
    type Error = crate::Error;
    type Filter = Condition;

    async fn find(&self, id: WorkspaceRoleId) -> Result<Option<Root<WorkspaceRole>>, crate::Error> {
        self.find_by(Condition::all().add(Expr::col("id").eq(id.to_string())))
            .await
    }

    async fn find_by(
        &self,
        filter: Condition,
    ) -> Result<Option<Root<WorkspaceRole>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let row = rm.fetch_optional_row(&stmt).await?;
        row.map(|r| self.row_to_root(r)).transpose()
    }

    async fn find_many(
        &self,
        ids: Vec<WorkspaceRoleId>,
    ) -> Result<Vec<Root<WorkspaceRole>>, crate::Error> {
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
    ) -> Result<Vec<Root<WorkspaceRole>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let rows = rm.fetch_all_rows(&stmt).await?;
        rows.into_iter().map(|row| self.row_to_root(row)).collect()
    }

    async fn all(&self) -> Result<Vec<Root<WorkspaceRole>>, crate::Error> {
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
impl WriteRepository<WorkspaceRole> for WorkspaceRoleRepository {
    type Error = crate::Error;
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

#[async_trait]
impl WorkspaceRoleRepositoryTrait<AnyRow> for WorkspaceRoleRepository {
    type Error = crate::Error;

    async fn count_members_with_role(
        &self,
        role_id: &WorkspaceRoleId,
    ) -> Result<u64, crate::Error> {
        let row = sqlx::query(
            "SELECT COUNT(*) as cnt FROM projections__workspace_user_roles WHERE workspace_role_id = ?",
        )
        .bind(role_id.to_string())
        .fetch_one(self.store.pool.as_ref())
        .await?;
        let count: i64 = row.try_get(0)?;
        Ok(count as u64)
    }

    async fn find_with_permissions(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<WorkspaceRoleWithPermissionsRow>, crate::Error> {
        let workspace_id_str = workspace_id.to_string();
        let sql = match self.store.pool.database_type() {
            crate::DatabaseType::Sqlite => {
                "SELECT wr.id, wr.workspace_id, wr.name, \
                 GROUP_CONCAT(DISTINCT wrp.permission_id) AS perm_ids, \
                 GROUP_CONCAT(DISTINCT p.name) AS perm_names \
                 FROM projections__workspace_roles wr \
                 LEFT JOIN projections__workspace_role_permissions wrp ON wr.id = wrp.workspace_role_id \
                 LEFT JOIN permissions p ON wrp.permission_id = p.id \
                 WHERE wr.workspace_id = ? \
                 GROUP BY wr.id, wr.workspace_id, wr.name"
            }
            crate::DatabaseType::Postgres => {
                "SELECT wr.id, wr.workspace_id, wr.name, \
                 STRING_AGG(DISTINCT wrp.permission_id::text, ',') AS perm_ids, \
                 STRING_AGG(DISTINCT p.name, ',') AS perm_names \
                 FROM projections__workspace_roles wr \
                 LEFT JOIN projections__workspace_role_permissions wrp ON wr.id = wrp.workspace_role_id \
                 LEFT JOIN permissions p ON wrp.permission_id = p.id \
                 WHERE wr.workspace_id = $1 \
                 GROUP BY wr.id, wr.workspace_id, wr.name"
            }
        };

        let rows = sqlx::query(sql)
            .bind(&workspace_id_str)
            .fetch_all(self.store.pool.as_ref())
            .await?;

        rows.into_iter()
            .map(|row| {
                let id: String = row.try_get("id")?;
                let ws_id: String = row.try_get("workspace_id")?;
                let name: Option<String> = row.try_get("name")?;
                let perm_ids_raw: Option<String> = row.try_get("perm_ids")?;
                let perm_names_raw: Option<String> = row.try_get("perm_names")?;
                let permission_ids = perm_ids_raw
                    .unwrap_or_default()
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(ToOwned::to_owned)
                    .collect();
                let permission_names = perm_names_raw
                    .unwrap_or_default()
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(ToOwned::to_owned)
                    .collect();
                Ok(WorkspaceRoleWithPermissionsRow::new(
                    id,
                    ws_id,
                    name,
                    permission_ids,
                    permission_names,
                ))
            })
            .collect::<Result<Vec<_>, crate::Error>>()
    }
}
