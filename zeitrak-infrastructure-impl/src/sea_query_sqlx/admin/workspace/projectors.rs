use async_trait::async_trait;
use eventually_projection::{Projector, RawEvent};
use sea_query::{
    Condition, DynIden, Expr, ExprTrait, OnConflict, PostgresQueryBuilder, Query,
    SqliteQueryBuilder, TableRef,
};
use sea_query_sqlx::SqlxBinder;
use zeitrak_core::admin::workspace::WorkspaceEvent;

use crate::{DatabaseType, Pool, ScopeAdmin, StateConnected};

pub struct WorkspaceProjector {
    pool: Pool<ScopeAdmin, StateConnected>,
}

impl WorkspaceProjector {
    const TABLE: &'static str = "projections__workspaces";
    const USER_ROLES_TABLE: &'static str = "projections__workspace_user_roles";
    const USER_PERMISSIONS_TABLE: &'static str = "projections__workspace_user_permissions";

    #[must_use]
    pub const fn new(pool: Pool<ScopeAdmin, StateConnected>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Projector for WorkspaceProjector {
    type Error = crate::Error;

    #[allow(clippy::too_many_lines)]
    async fn handle(&mut self, event: RawEvent) -> Result<(), Self::Error> {
        match event.event_type.as_str() {
            "WorkspaceCreated" => {
                let WorkspaceEvent::Created { id, name } =
                    serde_json::from_slice(&event.payload_bytes)?
                else {
                    return Ok(());
                };

                let query = Query::insert()
                    .into_table(TableRef::from(Self::TABLE))
                    .columns([DynIden::from("id"), DynIden::from("name")])
                    .values_panic([id.to_string().into(), sea_query::Value::String(name).into()])
                    .on_conflict(OnConflict::new().do_nothing().to_owned())
                    .to_owned();

                let (sql, values) = match self.pool.database_type() {
                    DatabaseType::Sqlite => query.build_sqlx(SqliteQueryBuilder),
                    DatabaseType::Postgres => query.build_sqlx(PostgresQueryBuilder),
                };

                sqlx::query_with(&sql, values)
                    .execute(self.pool.as_ref())
                    .await?;
            }
            "WorkspaceUserRoleAssigned" => {
                let WorkspaceEvent::UserRoleAssigned {
                    user_id,
                    workspace_role_id,
                } = serde_json::from_slice(&event.payload_bytes)?
                else {
                    return Ok(());
                };

                let query = Query::insert()
                    .into_table(TableRef::from(Self::USER_ROLES_TABLE))
                    .columns([
                        DynIden::from("workspace_id"),
                        DynIden::from("user_id"),
                        DynIden::from("workspace_role_id"),
                    ])
                    .values_panic([
                        event.stream_id.clone().into(),
                        user_id.to_string().into(),
                        workspace_role_id.to_string().into(),
                    ])
                    .on_conflict(OnConflict::new().do_nothing().to_owned())
                    .to_owned();

                let (sql, values) = match self.pool.database_type() {
                    DatabaseType::Sqlite => query.build_sqlx(SqliteQueryBuilder),
                    DatabaseType::Postgres => query.build_sqlx(PostgresQueryBuilder),
                };

                sqlx::query_with(&sql, values)
                    .execute(self.pool.as_ref())
                    .await?;
            }
            "WorkspaceUserRoleRevoked" => {
                let WorkspaceEvent::UserRoleRevoked {
                    user_id,
                    workspace_role_id,
                } = serde_json::from_slice(&event.payload_bytes)?
                else {
                    return Ok(());
                };

                let query = Query::delete()
                    .from_table(TableRef::from(Self::USER_ROLES_TABLE))
                    .cond_where(
                        Condition::all()
                            .add(Expr::col("workspace_id").eq(Expr::val(event.stream_id.clone())))
                            .add(Expr::col("user_id").eq(Expr::val(user_id.to_string())))
                            .add(
                                Expr::col("workspace_role_id")
                                    .eq(Expr::val(workspace_role_id.to_string())),
                            ),
                    )
                    .to_owned();

                let (sql, values) = match self.pool.database_type() {
                    DatabaseType::Sqlite => query.build_sqlx(SqliteQueryBuilder),
                    DatabaseType::Postgres => query.build_sqlx(PostgresQueryBuilder),
                };

                sqlx::query_with(&sql, values)
                    .execute(self.pool.as_ref())
                    .await?;
            }
            "WorkspaceUserPermissionGranted" => {
                let WorkspaceEvent::UserPermissionGranted {
                    user_id,
                    permission_id,
                } = serde_json::from_slice(&event.payload_bytes)?
                else {
                    return Ok(());
                };

                let query = Query::insert()
                    .into_table(TableRef::from(Self::USER_PERMISSIONS_TABLE))
                    .columns([
                        DynIden::from("workspace_id"),
                        DynIden::from("user_id"),
                        DynIden::from("permission_id"),
                    ])
                    .values_panic([
                        event.stream_id.clone().into(),
                        user_id.to_string().into(),
                        permission_id.to_string().into(),
                    ])
                    .on_conflict(OnConflict::new().do_nothing().to_owned())
                    .to_owned();

                let (sql, values) = match self.pool.database_type() {
                    DatabaseType::Sqlite => query.build_sqlx(SqliteQueryBuilder),
                    DatabaseType::Postgres => query.build_sqlx(PostgresQueryBuilder),
                };

                sqlx::query_with(&sql, values)
                    .execute(self.pool.as_ref())
                    .await?;
            }
            "WorkspaceUserPermissionRevoked" => {
                let WorkspaceEvent::UserPermissionRevoked {
                    user_id,
                    permission_id,
                } = serde_json::from_slice(&event.payload_bytes)?
                else {
                    return Ok(());
                };

                let query = Query::delete()
                    .from_table(TableRef::from(Self::USER_PERMISSIONS_TABLE))
                    .cond_where(
                        Condition::all()
                            .add(Expr::col("workspace_id").eq(Expr::val(event.stream_id.clone())))
                            .add(Expr::col("user_id").eq(Expr::val(user_id.to_string())))
                            .add(
                                Expr::col("permission_id").eq(Expr::val(permission_id.to_string())),
                            ),
                    )
                    .to_owned();

                let (sql, values) = match self.pool.database_type() {
                    DatabaseType::Sqlite => query.build_sqlx(SqliteQueryBuilder),
                    DatabaseType::Postgres => query.build_sqlx(PostgresQueryBuilder),
                };

                sqlx::query_with(&sql, values)
                    .execute(self.pool.as_ref())
                    .await?;
            }
            "WorkspaceSettingsUpdated" => {
                let WorkspaceEvent::SettingsUpdated {
                    name,
                    timezone,
                    date_format,
                    currency,
                    week_start,
                } = serde_json::from_slice(&event.payload_bytes)?
                else {
                    return Ok(());
                };

                let query = Query::update()
                    .table(TableRef::from(Self::TABLE))
                    .values([
                        (DynIden::from("name"), sea_query::Value::String(name).into()),
                        (DynIden::from("timezone"), timezone.into()),
                        (DynIden::from("date_format"), date_format.into()),
                        (DynIden::from("currency"), currency.into()),
                        (DynIden::from("week_start"), week_start.into()),
                    ])
                    .cond_where(
                        Condition::all()
                            .add(Expr::col("id").eq(Expr::val(event.stream_id.clone()))),
                    )
                    .to_owned();

                let (sql, values) = match self.pool.database_type() {
                    DatabaseType::Sqlite => query.build_sqlx(SqliteQueryBuilder),
                    DatabaseType::Postgres => query.build_sqlx(PostgresQueryBuilder),
                };

                sqlx::query_with(&sql, values)
                    .execute(self.pool.as_ref())
                    .await?;
            }
            _ => {}
        }

        Ok(())
    }
}
