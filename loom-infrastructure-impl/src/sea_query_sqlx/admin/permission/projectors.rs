use async_trait::async_trait;
use eventually_projection::{Projector, RawEvent};
use loom_core::admin::permission::PermissionEvent;
use sea_query::{DynIden, OnConflict, PostgresQueryBuilder, Query, SqliteQueryBuilder, TableRef};
use sea_query_sqlx::SqlxBinder;

use crate::{DatabaseType, Pool, ScopeAdmin, StateConnected};

pub struct PermissionProjector {
    pool: Pool<ScopeAdmin, StateConnected>,
}

impl PermissionProjector {
    const TABLE: &'static str = "permissions";

    #[must_use]
    pub const fn new(pool: Pool<ScopeAdmin, StateConnected>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Projector for PermissionProjector {
    type Error = crate::Error;

    async fn handle(&mut self, event: RawEvent) -> Result<(), Self::Error> {
        match event.event_type.as_str() {
            "PermissionCreated" => {
                let PermissionEvent::Created { id, name } =
                    serde_json::from_slice(&event.payload_bytes)?;

                let query = Query::insert()
                    .into_table(TableRef::from(Self::TABLE))
                    .columns([DynIden::from("id"), DynIden::from("name")])
                    .values_panic([id.to_string().into(), name.into()])
                    .on_conflict(OnConflict::new().do_nothing().to_owned())
                    .to_owned();

                let (sql, values) = match self.pool.database_type() {
                    DatabaseType::Sqlite => query.build_sqlx(SqliteQueryBuilder),
                    DatabaseType::Postgres => query.build_sqlx(PostgresQueryBuilder),
                };

                sqlx::query_with(&sql, values)
                    .execute(self.pool.as_ref())
                    .await?;

                Ok(())
            }
            _ => Ok(()),
        }
    }
}
