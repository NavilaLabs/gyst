use async_trait::async_trait;
use eventually_projection::{Projector, RawEvent};
use sea_query::{DynIden, OnConflict, PostgresQueryBuilder, Query, SqliteQueryBuilder, TableRef};
use sea_query_sqlx::SqlxBinder;
use zeitrak_core::admin::user::UserEvent;

use crate::{DatabaseType, Pool, ScopeAdmin, StateConnected};

pub struct UserProjector {
    pool: Pool<ScopeAdmin, StateConnected>,
}

impl UserProjector {
    const TABLE: &'static str = "projections__users";

    #[must_use]
    pub const fn new(pool: Pool<ScopeAdmin, StateConnected>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Projector for UserProjector {
    type Error = crate::Error;

    async fn handle(&mut self, event: RawEvent) -> Result<(), Self::Error> {
        match event.event_type.as_str() {
            "UserCreated" => {
                let UserEvent::Created {
                    id,
                    name,
                    email,
                    password,
                } = serde_json::from_slice(&event.payload_bytes)?
                else {
                    return Ok(());
                };

                let query = Query::insert()
                    .into_table(TableRef::from(Self::TABLE))
                    .columns([
                        DynIden::from("id"),
                        DynIden::from("name"),
                        DynIden::from("email"),
                        DynIden::from("password"),
                    ])
                    .values_panic([
                        id.to_string().into(),
                        name.into(),
                        email.into(),
                        password.into(),
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

                Ok(())
            }
            "UserSettingsUpdated" => {
                use sea_query::{Condition, Expr, ExprTrait, Query as SQ};

                let UserEvent::SettingsUpdated {
                    timezone,
                    date_format,
                    language,
                } = serde_json::from_slice(&event.payload_bytes)?
                else {
                    return Ok(());
                };
                let query = SQ::update()
                    .table(TableRef::from(Self::TABLE))
                    .values([
                        (DynIden::from("timezone"), timezone.into()),
                        (DynIden::from("date_format"), date_format.into()),
                        (DynIden::from("language"), language.into()),
                    ])
                    .cond_where(
                        Condition::all()
                            .add(Expr::col("id").eq(Expr::val(event.stream_id.clone()))),
                    )
                    .to_owned();

                let (sql, values) = match self.pool.database_type() {
                    DatabaseType::Sqlite => {
                        use sea_query_sqlx::SqlxBinder;
                        query.build_sqlx(SqliteQueryBuilder)
                    }
                    DatabaseType::Postgres => {
                        use sea_query_sqlx::SqlxBinder;
                        query.build_sqlx(PostgresQueryBuilder)
                    }
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
