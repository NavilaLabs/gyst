use std::{fmt::Debug, ops::Deref, str::FromStr};

use async_trait::async_trait;
use chrono::DateTime;
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::aggregate::{Aggregate, Root};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use sea_query::{Alias, Condition, Expr, ExprTrait};
use sqlx::{Row, any::AnyRow};
use zeitrak_core::admin::{
    invitation::{
        Invitation, InvitationEvent, InvitationId,
        InvitationRepository as InvitationRepositoryTrait, InvitationRow, InvitationStatus,
    },
    workspace::WorkspaceId,
    workspace_role::WorkspaceRoleId,
};
use zeitrak_core::shared::repositories::{ReadRepository, RowToRoot, WriteRepository};

use crate::{
    ConnectedAdminPool, infrastructure::read_model::SeaQueryReadModel, snapshot::SnapshotRepository,
};

const TABLE: &str = "projections__invitations";

pub struct InvitationRepository {
    store: SnapshotRepository<Invitation, ConnectedAdminPool>,
}

impl Debug for InvitationRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InvitationRepository")
            .finish_non_exhaustive()
    }
}

impl Deref for InvitationRepository {
    type Target = Repository<Invitation, Json<Invitation>, Json<InvitationEvent>>;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl InvitationRepository {
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
    ) -> &Repository<Invitation, Json<Invitation>, Json<InvitationEvent>> {
        self.store.event_store()
    }

    const fn read_model(&self) -> SeaQueryReadModel<'_> {
        SeaQueryReadModel::new(&self.store.pool, TABLE)
    }

    #[allow(clippy::unused_self)]
    fn row_to_invitation_row(&self, row: &AnyRow) -> Result<InvitationRow, crate::Error> {
        let id: String = row.try_get("id")?;
        let id = InvitationId::from_str(&id)?;
        let workspace_id: String = row.try_get("workspace_id")?;
        let workspace_id = WorkspaceId::from_str(&workspace_id)?;
        let workspace_name: Option<String> = row.try_get("workspace_name").ok();
        let email: String = row.try_get("email")?;
        let workspace_role_id: String = row.try_get("workspace_role_id")?;
        let workspace_role_id = WorkspaceRoleId::from_str(&workspace_role_id)?;
        let token: String = row.try_get("token")?;
        let status_str: String = row.try_get("status")?;
        let status = match status_str.as_str() {
            "accepted" => InvitationStatus::Accepted,
            "revoked" => InvitationStatus::Revoked,
            _ => InvitationStatus::Pending,
        };
        let expires_at_str: String = row.try_get("expires_at")?;
        let expires_at = expires_at_str.parse::<DateTime<chrono::Utc>>()?;
        Ok(InvitationRow::new(
            id,
            workspace_id,
            workspace_name,
            email,
            workspace_role_id,
            token,
            status,
            expires_at,
        ))
    }
}

impl RowToRoot<AnyRow, Invitation> for InvitationRepository {
    type Error = crate::Error;

    fn row_to_root(&self, row: AnyRow) -> Result<Root<Invitation>, crate::Error> {
        let id: String = row.try_get("id")?;
        let id = InvitationId::from_str(&id)?;
        let workspace_id: String = row.try_get("workspace_id")?;
        let workspace_id = WorkspaceId::from_str(&workspace_id)?;
        let invited_by: String = row.try_get("invited_by")?;
        let invited_by = zeitrak_core::admin::user::UserId::from_str(&invited_by)?;
        let email: String = row.try_get("email")?;
        let workspace_role_id: String = row.try_get("workspace_role_id")?;
        let workspace_role_id = WorkspaceRoleId::from_str(&workspace_role_id)?;
        let token: String = row.try_get("token")?;
        let expires_at_str: String = row.try_get("expires_at")?;
        let expires_at = expires_at_str.parse::<DateTime<chrono::Utc>>()?;
        let invitation = Invitation::apply(
            None,
            InvitationEvent::Created {
                id,
                workspace_id,
                invited_by,
                email,
                workspace_role_id,
                token,
                expires_at,
            },
        )
        .expect("Created event on None state is infallible");
        Ok(Root::rehydrate_from_state(0, invitation))
    }
}

impl zeitrak_core::shared::repositories::Repository<Invitation, AnyRow> for InvitationRepository {}

#[async_trait]
impl ReadRepository<Invitation, AnyRow> for InvitationRepository {
    type Error = crate::Error;
    type Filter = Condition;

    async fn find(&self, id: InvitationId) -> Result<Option<Root<Invitation>>, crate::Error> {
        self.find_by(Condition::all().add(Expr::col("id").eq(id.to_string())))
            .await
    }

    async fn find_by(&self, filter: Condition) -> Result<Option<Root<Invitation>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let row = rm.fetch_optional_row(&stmt).await?;
        row.map(|r| self.row_to_root(r)).transpose()
    }

    async fn find_many(
        &self,
        ids: Vec<InvitationId>,
    ) -> Result<Vec<Root<Invitation>>, crate::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let id_strings: Vec<String> = ids.into_iter().map(|id| id.to_string()).collect();
        self.find_many_by(Condition::all().add(Expr::col("id").is_in(id_strings)))
            .await
    }

    async fn find_many_by(&self, filter: Condition) -> Result<Vec<Root<Invitation>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let rows = rm.fetch_all_rows(&stmt).await?;
        rows.into_iter().map(|row| self.row_to_root(row)).collect()
    }

    async fn all(&self) -> Result<Vec<Root<Invitation>>, crate::Error> {
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
impl WriteRepository<Invitation> for InvitationRepository {
    type Error = crate::Error;
}

#[async_trait]
impl Getter<Invitation> for InvitationRepository {
    async fn get(&self, id: &InvitationId) -> Result<Root<Invitation>, GetError> {
        self.store.get(id).await
    }
}

#[async_trait]
impl Saver<Invitation> for InvitationRepository {
    async fn save(&self, root: &mut Root<Invitation>) -> Result<(), SaveError> {
        self.store.save(root).await
    }
}

#[async_trait]
impl InvitationRepositoryTrait<AnyRow> for InvitationRepository {
    type Error = crate::Error;

    async fn find_by_token(&self, token: &str) -> Result<Option<InvitationRow>, crate::Error> {
        let row = sqlx::query(
            "SELECT i.id, i.workspace_id, w.name AS workspace_name, i.email, \
             i.workspace_role_id, i.token, i.status, i.expires_at \
             FROM projections__invitations i \
             LEFT JOIN projections__workspaces w ON i.workspace_id = w.id \
             WHERE i.token = ?",
        )
        .bind(token)
        .fetch_optional(self.store.pool.as_ref())
        .await?;
        row.map(|r| self.row_to_invitation_row(&r)).transpose()
    }

    async fn find_by_workspace(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<InvitationRow>, crate::Error> {
        let rows = sqlx::query(
            "SELECT id, workspace_id, NULL AS workspace_name, email, workspace_role_id, \
             token, status, expires_at \
             FROM projections__invitations \
             WHERE workspace_id = ?",
        )
        .bind(workspace_id.to_string())
        .fetch_all(self.store.pool.as_ref())
        .await?;
        rows.iter().map(|r| self.row_to_invitation_row(r)).collect()
    }

    async fn find_pending_for_email(&self, email: &str) -> Result<Vec<InvitationId>, crate::Error> {
        let statement = sea_query::Query::select()
            .expr(Expr::col(Alias::new("id")))
            .from(Alias::new(TABLE))
            .and_where(Expr::col(Alias::new("email")).eq(email))
            .and_where(Expr::col(Alias::new("status")).eq("pending"))
            .to_owned();
        let (sql, arguments) = self.store.pool.build_query(&statement);
        let rows = sqlx::query_with(&sql, arguments)
            .fetch_all(self.store.pool.as_ref())
            .await?;
        rows.into_iter()
            .map(|r| {
                let id: String = r.try_get(0usize)?;
                InvitationId::from_str(&id).map_err(crate::Error::from)
            })
            .collect()
    }

    async fn find_all_pending_for_email(
        &self,
        email: &str,
    ) -> Result<Vec<InvitationRow>, crate::Error> {
        let rows = sqlx::query(
            "SELECT i.id, i.workspace_id, w.name AS workspace_name, i.email, \
             i.workspace_role_id, i.token, i.status, i.expires_at \
             FROM projections__invitations i \
             LEFT JOIN projections__workspaces w ON i.workspace_id = w.id \
             WHERE i.email = ? AND i.status = 'pending'",
        )
        .bind(email)
        .fetch_all(self.store.pool.as_ref())
        .await?;
        rows.iter().map(|r| self.row_to_invitation_row(r)).collect()
    }
}
