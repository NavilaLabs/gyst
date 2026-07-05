use async_trait::async_trait;
use eventually_projection::{Projector, RawEvent};
use sea_query::{Condition, DynIden, Expr, ExprTrait, OnConflict, Query, TableRef};
use sea_query_sqlx::SqlxBinder;
use sqlx::AssertSqlSafe;
use zeitrak_core::tenant::activity::ActivityEvent;

use crate::{ConnectedTenantPool, DatabaseType};

pub struct ActivityProjector {
    pool: ConnectedTenantPool,
}

impl ActivityProjector {
    const TABLE: &'static str = "projections__activities";

    #[must_use]
    pub const fn new(pool: ConnectedTenantPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Projector for ActivityProjector {
    type Error = crate::Error;

    async fn handle(&mut self, event: RawEvent) -> Result<(), Self::Error> {
        match event.event_type.as_str() {
            "ActivityCreated" => {
                let ActivityEvent::Created {
                    id,
                    name,
                    color,
                    comment,
                } = serde_json::from_slice(&event.payload_bytes)?
                else {
                    return Ok(());
                };

                let query = Query::insert()
                    .into_table(TableRef::from(Self::TABLE))
                    .columns([
                        DynIden::from("id"),
                        DynIden::from("name"),
                        DynIden::from("color"),
                        DynIden::from("comment"),
                    ])
                    .values_panic([
                        id.to_string().into(),
                        name.into(),
                        color.into(),
                        comment.into(),
                    ])
                    .on_conflict(OnConflict::new().do_nothing().to_owned())
                    .to_owned();

                let (sql, values) = self.pool.build_query(&query);
                sqlx::query_with(AssertSqlSafe(sql.as_str()), values)
                    .execute(self.pool.as_ref())
                    .await?;
            }
            "ActivityUpdated" => {
                let ActivityEvent::Updated {
                    name,
                    color,
                    comment,
                } = serde_json::from_slice(&event.payload_bytes)?
                else {
                    return Ok(());
                };

                let query = Query::update()
                    .table(TableRef::from(Self::TABLE))
                    .values([
                        (DynIden::from("name"), name.into()),
                        (DynIden::from("color"), color.into()),
                        (DynIden::from("comment"), comment.into()),
                    ])
                    .cond_where(
                        Condition::all()
                            .add(Expr::col("id").eq(Expr::val(event.stream_id.clone()))),
                    )
                    .to_owned();

                let (sql, values) = match self.pool.database_type() {
                    DatabaseType::Sqlite => query.build_sqlx(sea_query::SqliteQueryBuilder),
                    DatabaseType::Postgres => query.build_sqlx(sea_query::PostgresQueryBuilder),
                };
                sqlx::query_with(AssertSqlSafe(sql.as_str()), values)
                    .execute(self.pool.as_ref())
                    .await?;
            }
            "ActivityDeleted" => {
                sqlx::query(
                    "UPDATE projections__activities SET deleted_at = CURRENT_TIMESTAMP WHERE id = ?",
                )
                .bind(event.stream_id.clone())
                .execute(self.pool.as_ref())
                .await?;
            }
            _ => {}
        }

        Ok(())
    }
}
