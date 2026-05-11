use std::fmt::Debug;
use std::{ops::Deref, str::FromStr};

use async_trait::async_trait;
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::aggregate::{Aggregate, Root};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use sea_query::{Alias, Condition, Expr, ExprTrait};
use sqlx::{Row, any::AnyRow};
use zeitrak_core::admin::user::{
    User, UserEvent, UserId, UserRepository as UserRepositoryTrait, UserRow,
};
use zeitrak_core::shared::repositories::{ReadRepository, RowToRoot, WriteRepository};

use crate::{
    ConnectedAdminPool, infrastructure::read_model::SeaQueryReadModel, snapshot::SnapshotRepository,
};

const TABLE: &str = "projections__users";

pub struct UserRepository {
    store: SnapshotRepository<User, ConnectedAdminPool>,
}

impl Debug for UserRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserRepository")
            .field("pool", &self.store.pool)
            .field("store", &self.store.event_store())
            .finish()
    }
}

impl Deref for UserRepository {
    type Target = Repository<User, Json<User>, Json<UserEvent>>;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl UserRepository {
    /// # Errors
    ///
    /// Returns an error if the event store repository cannot be initialized.
    pub async fn from_pool(pool: ConnectedAdminPool) -> Result<Self, sqlx::migrate::MigrateError> {
        Ok(Self {
            store: SnapshotRepository::from_pool(pool).await?,
        })
    }

    #[must_use]
    pub const fn event_store(&self) -> &Repository<User, Json<User>, Json<UserEvent>> {
        self.store.event_store()
    }

    const fn read_model(&self) -> SeaQueryReadModel<'_> {
        SeaQueryReadModel::new(&self.store.pool, TABLE)
    }

    #[allow(clippy::unused_self)]
    fn row_to_view(&self, row: &AnyRow) -> Result<UserRow, crate::Error> {
        let id: String = row.try_get("id")?;
        let id = UserId::from_str(&id)?;
        let name: String = row.try_get("name")?;
        let email: String = row.try_get("email")?;
        let timezone: String = row
            .try_get("timezone")
            .unwrap_or_else(|_| "Europe/Berlin".to_string());
        let date_format: String = row
            .try_get("date_format")
            .unwrap_or_else(|_| "%Y-%m-%d".to_string());
        let language: String = row.try_get("language").unwrap_or_else(|_| "en".to_string());
        Ok(UserRow::new_with_settings(
            id,
            name,
            email,
            timezone,
            date_format,
            language,
        ))
    }

    async fn find_view_by_id_impl(&self, id: &str) -> Result<Option<UserRow>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm
            .select()
            .and_where(Expr::col(Alias::new("id")).eq(id))
            .to_owned();
        let row = rm.fetch_optional_row(&stmt).await?;
        row.map(|r| self.row_to_view(&r)).transpose()
    }
}

#[async_trait]
impl Getter<User> for UserRepository {
    async fn get(&self, id: &UserId) -> Result<Root<User>, GetError> {
        self.store.get(id).await
    }
}

#[async_trait]
impl Saver<User> for UserRepository {
    async fn save(&self, root: &mut Root<User>) -> Result<(), SaveError> {
        self.store.save(root).await
    }
}

impl RowToRoot<AnyRow, User> for UserRepository {
    type Error = crate::Error;

    fn row_to_root(&self, row: AnyRow) -> Result<Root<User>, crate::Error> {
        let id: String = row.try_get("id")?;
        let id = UserId::from_str(&id)?;
        let name: String = row.try_get("name")?;
        let email: String = row.try_get("email")?;
        let password: String = row.try_get("password")?;
        let timezone: String = row
            .try_get("timezone")
            .unwrap_or_else(|_| "Europe/Berlin".to_string());
        let date_format: String = row
            .try_get("date_format")
            .unwrap_or_else(|_| "%Y-%m-%d".to_string());
        let language: String = row.try_get("language").unwrap_or_else(|_| "en".to_string());
        let user = User::apply(
            None,
            UserEvent::Created {
                id,
                name,
                email,
                password,
            },
        )
        .expect("Created event on None state is infallible");
        let user = User::apply(
            Some(user),
            UserEvent::SettingsUpdated {
                timezone,
                date_format,
                language,
            },
        )
        .expect("SettingsUpdated event on Some state is infallible");
        Ok(Root::rehydrate_from_state(0, user)) // TODO: really get the version of the aggregte root.
    }
}

impl zeitrak_core::shared::repositories::Repository<User, AnyRow> for UserRepository {}

#[async_trait]
impl ReadRepository<User, AnyRow> for UserRepository {
    type Error = crate::Error;
    type Filter = Condition;

    async fn find(&self, id: UserId) -> Result<Option<Root<User>>, crate::Error> {
        self.find_by(Condition::all().add(Expr::col("id").eq(id.to_string())))
            .await
    }

    async fn find_by(&self, filter: Condition) -> Result<Option<Root<User>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let row = rm.fetch_optional_row(&stmt).await?;
        row.map(|r| self.row_to_root(r)).transpose()
    }

    async fn find_many(&self, ids: Vec<UserId>) -> Result<Vec<Root<User>>, crate::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let id_strings: Vec<String> = ids.into_iter().map(|id| id.to_string()).collect();
        self.find_many_by(Condition::all().add(Expr::col("id").is_in(id_strings)))
            .await
    }

    async fn find_many_by(&self, filter: Condition) -> Result<Vec<Root<User>>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let rows = rm.fetch_all_rows(&stmt).await?;
        rows.into_iter().map(|row| self.row_to_root(row)).collect()
    }

    async fn all(&self) -> Result<Vec<Root<User>>, crate::Error> {
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
impl WriteRepository<User> for UserRepository {
    type Error = crate::Error;
}

#[async_trait]
impl UserRepositoryTrait<AnyRow> for UserRepository {
    type Error = crate::Error;

    async fn find_view_by_id(&self, id: &str) -> Result<Option<UserRow>, crate::Error> {
        self.find_view_by_id_impl(id).await
    }

    async fn find_id_by_email(&self, email: &str) -> Result<Option<UserId>, crate::Error> {
        let statement = sea_query::Query::select()
            .expr(Expr::col(Alias::new("id")))
            .from(Alias::new(TABLE))
            .and_where(Expr::col(Alias::new("email")).eq(email))
            .to_owned();
        let (sql, arguments) = self.store.pool.build_query(&statement);
        let row = sqlx::query_with(&sql, arguments)
            .fetch_optional(self.store.pool.as_ref())
            .await?;
        row.map(|r| {
            let id: String = r.try_get(0usize)?;
            UserId::from_str(&id).map_err(Into::into)
        })
        .transpose()
    }

    async fn find_credentials_by_email(
        &self,
        email: &str,
    ) -> Result<Option<(String, String, String)>, crate::Error> {
        let statement = sea_query::Query::select()
            .expr(Expr::col(Alias::new("id")))
            .expr(Expr::col(Alias::new("email")))
            .expr(Expr::col(Alias::new("password")))
            .from(Alias::new(TABLE))
            .and_where(Expr::col(Alias::new("email")).eq(email))
            .to_owned();
        let (sql, arguments) = self.store.pool.build_query(&statement);

        tracing::debug!(sql = %sql, "find_credentials_by_email");

        let row = sqlx::query_with(&sql, arguments)
            .fetch_optional(self.store.pool.as_ref())
            .await?;

        row.map(|r| {
            let id: String = r.try_get(0usize)?;
            let email: String = r.try_get(1usize)?;
            let hash: String = r.try_get(2usize)?;
            Ok((id, email, hash))
        })
        .transpose()
    }
}
