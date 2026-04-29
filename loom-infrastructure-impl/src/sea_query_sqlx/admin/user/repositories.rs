use std::{ops::Deref, str::FromStr};

use async_trait::async_trait;
use eventually::aggregate::Root;
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use loom_core::admin::user::{
    User, UserEvent, UserId, UserRepository as UserRepositoryTrait, UserRow,
};
use loom_infrastructure::repository::{EntryToRow, ReadRepository};
use sea_query::{Alias, Condition, Expr, ExprTrait};
use sqlx::{Row, any::AnyRow, types::Uuid};

use crate::{
    ConnectedAdminPool, infrastructure::read_model::SeaQueryReadModel, snapshot::SnapshotRepository,
};

const TABLE: &str = "projections__users";

pub struct UserRepository {
    store: SnapshotRepository<User, ConnectedAdminPool>,
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

    /// Returns `(user_id, email, password)` for the given email — intended
    /// only for authentication flows, not general display.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_credentials_by_email(
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

    /// # Errors
    ///
    /// Returns an error if the database count query fails.
    pub async fn has_at_least_one_user(&self) -> Result<bool, crate::Error> {
        Ok(self.count().await? > 0)
    }

    /// Fetch a `UserView` by string ID, avoiding the `AnyPool` UUID-type panic.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_view_by_id(&self, id: &str) -> Result<Option<UserRow>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm
            .select()
            .and_where(Expr::col(Alias::new("id")).eq(id))
            .to_owned();
        let row = rm.fetch_optional_row(&stmt).await?;
        row.map(|r| self.entry_to_row(r)).transpose()
    }
}

impl EntryToRow<AnyRow> for UserRepository {
    type Row = UserRow;
    type Error = crate::Error;

    fn entry_to_row(&self, row: AnyRow) -> Result<UserRow, crate::Error> {
        let id: String = row.try_get("id")?;
        let id = Uuid::from_str(&id)?;
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
            id.into(),
            name,
            email,
            timezone,
            date_format,
            language,
        ))
    }
}

#[async_trait]
impl ReadRepository<AnyRow> for UserRepository {
    type Filter = Condition;

    async fn get_one(&self, id: Uuid) -> Result<UserRow, crate::Error> {
        self.get_one_by(Condition::all().add(Expr::col("id").eq(id)))
            .await
    }

    async fn find_one(&self, id: Uuid) -> Result<Option<UserRow>, crate::Error> {
        self.find_by(Condition::all().add(Expr::col("id").eq(id)))
            .await
    }

    async fn get_one_by(&self, filter: Condition) -> Result<UserRow, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let row = rm.fetch_one_row(&stmt).await?;
        self.entry_to_row(row)
    }

    async fn find_by(&self, filter: Condition) -> Result<Option<UserRow>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let row = rm.fetch_optional_row(&stmt).await?;
        row.map(|r| self.entry_to_row(r)).transpose()
    }

    async fn find_many(&self, ids: Vec<Uuid>) -> Result<Vec<UserRow>, crate::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        self.find_many_by(Condition::all().add(Expr::col("id").is_in(ids)))
            .await
    }

    async fn find_many_by(&self, filter: Condition) -> Result<Vec<UserRow>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let rows = rm.fetch_all_rows(&stmt).await?;
        rows.into_iter().map(|row| self.entry_to_row(row)).collect()
    }

    async fn all(&self) -> Result<Vec<UserRow>, crate::Error> {
        let rm = self.read_model();
        let stmt = rm.select();
        let rows = rm.fetch_all_rows(&stmt).await?;
        rows.into_iter().map(|row| self.entry_to_row(row)).collect()
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

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    type Error = crate::Error;

    async fn find_credentials_by_email(
        &self,
        email: &str,
    ) -> Result<Option<(String, String, String)>, Self::Error> {
        self.find_credentials_by_email(email).await
    }
}
