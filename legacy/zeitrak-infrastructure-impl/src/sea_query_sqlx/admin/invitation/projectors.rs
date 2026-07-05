use async_trait::async_trait;
use eventually_projection::{Projector, RawEvent};
use sea_query::{
    Condition, DynIden, Expr, ExprTrait, OnConflict, PostgresQueryBuilder, Query,
    SqliteQueryBuilder, TableRef,
};
use sea_query_sqlx::SqlxBinder;
use sqlx::AssertSqlSafe;
use zeitrak_core::admin::invitation::InvitationEvent;

use crate::{DatabaseType, Pool, ScopeAdmin, StateConnected};

pub struct InvitationProjector {
    pool: Pool<ScopeAdmin, StateConnected>,
}

impl InvitationProjector {
    const TABLE: &'static str = "projections__invitations";

    #[must_use]
    pub const fn new(pool: Pool<ScopeAdmin, StateConnected>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Projector for InvitationProjector {
    type Error = crate::Error;

    async fn handle(&mut self, event: RawEvent) -> Result<(), Self::Error> {
        match event.event_type.as_str() {
            "InvitationCreated" => {
                let InvitationEvent::Created {
                    id,
                    workspace_id,
                    invited_by,
                    email,
                    workspace_role_id,
                    token,
                    expires_at,
                } = serde_json::from_slice(&event.payload_bytes)?
                else {
                    return Ok(());
                };

                let query = Query::insert()
                    .into_table(TableRef::from(Self::TABLE))
                    .columns([
                        DynIden::from("id"),
                        DynIden::from("workspace_id"),
                        DynIden::from("invited_by"),
                        DynIden::from("email"),
                        DynIden::from("workspace_role_id"),
                        DynIden::from("token"),
                        DynIden::from("status"),
                        DynIden::from("expires_at"),
                    ])
                    .values_panic([
                        id.to_string().into(),
                        workspace_id.to_string().into(),
                        invited_by.to_string().into(),
                        email.into(),
                        workspace_role_id.to_string().into(),
                        token.into(),
                        "pending".into(),
                        expires_at.to_rfc3339().into(),
                    ])
                    .on_conflict(OnConflict::new().do_nothing().to_owned())
                    .to_owned();

                let (sql, values) = match self.pool.database_type() {
                    DatabaseType::Sqlite => query.build_sqlx(SqliteQueryBuilder),
                    DatabaseType::Postgres => query.build_sqlx(PostgresQueryBuilder),
                };

                sqlx::query_with(AssertSqlSafe(sql.as_str()), values)
                    .execute(self.pool.as_ref())
                    .await?;
            }
            "InvitationAccepted" => {
                let query = Query::update()
                    .table(TableRef::from(Self::TABLE))
                    .values([(DynIden::from("status"), "accepted".into())])
                    .cond_where(
                        Condition::all()
                            .add(Expr::col("id").eq(Expr::val(event.stream_id.clone()))),
                    )
                    .to_owned();

                let (sql, values) = match self.pool.database_type() {
                    DatabaseType::Sqlite => query.build_sqlx(SqliteQueryBuilder),
                    DatabaseType::Postgres => query.build_sqlx(PostgresQueryBuilder),
                };

                sqlx::query_with(AssertSqlSafe(sql.as_str()), values)
                    .execute(self.pool.as_ref())
                    .await?;
            }
            "InvitationRevoked" => {
                let query = Query::update()
                    .table(TableRef::from(Self::TABLE))
                    .values([(DynIden::from("status"), "revoked".into())])
                    .cond_where(
                        Condition::all()
                            .add(Expr::col("id").eq(Expr::val(event.stream_id.clone()))),
                    )
                    .to_owned();

                let (sql, values) = match self.pool.database_type() {
                    DatabaseType::Sqlite => query.build_sqlx(SqliteQueryBuilder),
                    DatabaseType::Postgres => query.build_sqlx(PostgresQueryBuilder),
                };

                sqlx::query_with(AssertSqlSafe(sql.as_str()), values)
                    .execute(self.pool.as_ref())
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}
