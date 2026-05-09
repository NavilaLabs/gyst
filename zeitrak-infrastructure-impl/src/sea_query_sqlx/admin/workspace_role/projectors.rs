use async_trait::async_trait;
use eventually_projection::{Projector, RawEvent};
use zeitrak_core::admin::workspace_role::WorkspaceRoleEvent;
use sea_query::{
    Condition, DynIden, Expr, ExprTrait, OnConflict, PostgresQueryBuilder, Query,
    SqliteQueryBuilder, TableRef,
};
use sea_query_sqlx::SqlxBinder;

use crate::{DatabaseType, Pool, ScopeAdmin, StateConnected};

pub struct WorkspaceRoleProjector {
    pool: Pool<ScopeAdmin, StateConnected>,
}

impl WorkspaceRoleProjector {
    const TABLE: &'static str = "projections__workspace_roles";
    const PERMISSIONS_TABLE: &'static str = "projections__workspace_role_permissions";

    #[must_use]
    pub const fn new(pool: Pool<ScopeAdmin, StateConnected>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Projector for WorkspaceRoleProjector {
    type Error = crate::Error;

    async fn handle(&mut self, event: RawEvent) -> Result<(), Self::Error> {
        match event.event_type.as_str() {
            "WorkspaceRoleCreated" => {
                let WorkspaceRoleEvent::Created {
                    id,
                    workspace_id,
                    name,
                } = serde_json::from_slice(&event.payload_bytes)?
                else {
                    return Ok(());
                };

                let query = Query::insert()
                    .into_table(TableRef::from(Self::TABLE))
                    .columns([
                        DynIden::from("id"),
                        DynIden::from("workspace_id"),
                        DynIden::from("name"),
                    ])
                    .values_panic([
                        id.to_string().into(),
                        workspace_id.to_string().into(),
                        sea_query::Value::String(name).into(),
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
            "WorkspaceRolePermissionGranted" => {
                let WorkspaceRoleEvent::PermissionGranted { permission_id } =
                    serde_json::from_slice(&event.payload_bytes)?
                else {
                    return Ok(());
                };

                let query = Query::insert()
                    .into_table(TableRef::from(Self::PERMISSIONS_TABLE))
                    .columns([
                        DynIden::from("workspace_role_id"),
                        DynIden::from("permission_id"),
                    ])
                    .values_panic([
                        event.stream_id.clone().into(),
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
            "WorkspaceRolePermissionRevoked" => {
                let WorkspaceRoleEvent::PermissionRevoked { permission_id } =
                    serde_json::from_slice(&event.payload_bytes)?
                else {
                    return Ok(());
                };

                let query = Query::delete()
                    .from_table(TableRef::from(Self::PERMISSIONS_TABLE))
                    .cond_where(
                        Condition::all()
                            .add(
                                Expr::col("workspace_role_id")
                                    .eq(Expr::val(event.stream_id.clone())),
                            )
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
            _ => {}
        }

        Ok(())
    }
}
