use async_trait::async_trait;
use eventually_projection::{Projector, RawEvent};
use loom_core::tenant::timesheet_tag::TimesheetTagEvent;
use sea_query::{Condition, DynIden, Expr, ExprTrait, OnConflict, Query, TableRef};
use sea_query_sqlx::SqlxBinder;

use crate::{ConnectedTenantPool, DatabaseType};

pub struct TimesheetTagProjector {
    pool: ConnectedTenantPool,
}

impl TimesheetTagProjector {
    const TAGS_TABLE: &'static str = "projections__timesheet_tags";
    const TIMESHEET_TAGS_TABLE: &'static str = "projections__timesheet_has_tags";

    #[must_use]
    pub const fn new(pool: ConnectedTenantPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Projector for TimesheetTagProjector {
    type Error = crate::Error;

    async fn handle(&mut self, event: RawEvent) -> Result<(), Self::Error> {
        match event.event_type.as_str() {
            "TagCreated" => {
                let TimesheetTagEvent::Created { id, name } =
                    serde_json::from_slice(&event.payload_bytes)?
                else {
                    return Ok(());
                };

                let query = Query::insert()
                    .into_table(TableRef::from(Self::TAGS_TABLE))
                    .columns([DynIden::from("id"), DynIden::from("name")])
                    .values_panic([id.to_string().into(), name.into()])
                    .on_conflict(OnConflict::new().do_nothing().to_owned())
                    .to_owned();

                let (sql, values) = self.pool.build_query(&query);
                sqlx::query_with(&sql, values)
                    .execute(self.pool.as_ref())
                    .await?;
            }
            "TagRenamed" => {
                let TimesheetTagEvent::Renamed { name } =
                    serde_json::from_slice(&event.payload_bytes)?
                else {
                    return Ok(());
                };

                let query = Query::update()
                    .table(TableRef::from(Self::TAGS_TABLE))
                    .values([(DynIden::from("name"), name.into())])
                    .cond_where(
                        Condition::all()
                            .add(Expr::col("id").eq(Expr::val(event.stream_id.clone()))),
                    )
                    .to_owned();

                let (sql, values) = match self.pool.database_type() {
                    DatabaseType::Sqlite => query.build_sqlx(sea_query::SqliteQueryBuilder),
                    DatabaseType::Postgres => query.build_sqlx(sea_query::PostgresQueryBuilder),
                };
                sqlx::query_with(&sql, values)
                    .execute(self.pool.as_ref())
                    .await?;
            }
            "TagTimesheetTagged" => {
                let TimesheetTagEvent::TimesheetTagged { timesheet_id } =
                    serde_json::from_slice(&event.payload_bytes)?
                else {
                    return Ok(());
                };

                let query = Query::insert()
                    .into_table(TableRef::from(Self::TIMESHEET_TAGS_TABLE))
                    .columns([
                        DynIden::from("timesheet_id"),
                        DynIden::from("timesheet_tag_id"),
                    ])
                    .values_panic([
                        timesheet_id.to_string().into(),
                        event.stream_id.clone().into(),
                    ])
                    .on_conflict(OnConflict::new().do_nothing().to_owned())
                    .to_owned();

                let (sql, values) = self.pool.build_query(&query);
                sqlx::query_with(&sql, values)
                    .execute(self.pool.as_ref())
                    .await?;
            }
            "TagTimesheetUntagged" => {
                let TimesheetTagEvent::TimesheetUntagged { timesheet_id } =
                    serde_json::from_slice(&event.payload_bytes)?
                else {
                    return Ok(());
                };

                let query = Query::delete()
                    .from_table(TableRef::from(Self::TIMESHEET_TAGS_TABLE))
                    .cond_where(
                        Condition::all()
                            .add(Expr::col("timesheet_id").eq(Expr::val(timesheet_id.to_string())))
                            .add(
                                Expr::col("timesheet_tag_id")
                                    .eq(Expr::val(event.stream_id.clone())),
                            ),
                    )
                    .to_owned();

                let (sql, values) = match self.pool.database_type() {
                    DatabaseType::Sqlite => query.build_sqlx(sea_query::SqliteQueryBuilder),
                    DatabaseType::Postgres => query.build_sqlx(sea_query::PostgresQueryBuilder),
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
