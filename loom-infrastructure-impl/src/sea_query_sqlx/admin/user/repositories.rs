use std::{ops::Deref, str::FromStr};

use async_trait::async_trait;
use eventually::aggregate::Root;
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
use loom_core::admin::user::{
    User, UserEvent, UserId, UserRepository as UserRepositoryTrait, UserRow,
};
use loom_core::shared::repositories::{ReadRepository, WriteRepository};
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
impl ReadRepository<User> for UserRepository {
    type Error = crate::Error;
    type Filter = Condition;

    async fn find(&self, id: UserId) -> Result<Option<Root<User>>, Self::Error> {
        self.find_by(Condition::all().add(Expr::col("id").eq(id.to_string())))
            .await
    }

    async fn find_by(&self, filter: Condition) -> Result<Option<Root<User>>, Self::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let row = rm.fetch_optional_row(&stmt).await?;
        row.map(|r| self.entry_to_row(r)).transpose()
    }

    async fn find_many(&self, ids: Vec<UserId>) -> Result<Vec<Root<User>>, crate::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        self.find_many_by(Condition::all().add(Expr::col("id").is_in(ids)))
            .await
    }

    async fn find_many_by(&self, filter: Condition) -> Result<Vec<Root<User>>, Self::Error> {
        let rm = self.read_model();
        let stmt = rm.select().cond_where(filter).to_owned();
        let rows = rm.fetch_all_rows(&stmt).await?;
        rows.into_iter().map(|row| self.entry_to_row(row)).collect()
    }

    async fn all(&self) -> Result<Vec<Root<User>>, Self::Error> {
        let rm = self.read_model();
        let stmt = rm.select();
        let rows = rm.fetch_all_rows(&stmt).await?;
        rows.into_iter().map(|row| self.entry_to_row(row)).collect()
    }

    async fn count_by(&self, filter: Condition) -> Result<u64, Self::Error> {
        let rm = self.read_model();
        let stmt = rm.select_count().cond_where(filter).to_owned();
        rm.count_rows(&stmt).await
    }

    async fn count(&self) -> Result<u64, Self::Error> {
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
impl UserRepositoryTrait for UserRepository {
    type Error = crate::Error;

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
