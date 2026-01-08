#[cfg(feature = "sea-query-sqlx")]
pub mod sea_query_sqlx;

use std::fmt::Display;

use admin_migrations::{IntoSchemaManagerConnection, MigratorTrait};
use embassy_futures::join::join;
use log::info;

use crate::config::CONFIG;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database is already initialized")]
    AlreadyInitialized,
    #[error("Failed to initialize database")]
    FailedToInitialize,
    #[error("Failed to migrate database: {0}")]
    FailedToMigrate(#[from] shared_migrations::Error),
}

#[async_trait::async_trait]
pub trait Connection<T>
where
    T: Send + Sync,
{
    const DEFAULT_DATABASE_NAME: &'static str;
    type Error: Display + Send + Sync + From<Error>;

    async fn establish_admin_connection(&self) -> Result<T, Self::Error> {
        info!("Establishing connection to admin database");
        self.establish_connection(CONFIG.get_database().get_databases().get_admin().get_name())
            .await
    }

    async fn establish_tenant_connection(&self, tenant_token: &str) -> Result<T, Self::Error> {
        let tenant_db = format!(
            "{}{}",
            CONFIG
                .get_database()
                .get_databases()
                .get_tenant()
                .get_name_prefix(),
            tenant_token
        );
        info!("Establishing connection to tenant database: {tenant_db}");

        self.establish_connection(&tenant_db).await
    }

    async fn establish_default_connection(&self) -> Result<T, Self::Error> {
        self.establish_connection(Self::DEFAULT_DATABASE_NAME).await
    }

    async fn establish_connection(&self, database: &str) -> Result<T, Self::Error>;

    async fn close_connection(&self, pool: T);
}

#[async_trait::async_trait]
pub trait Initialize<Pool, T>
where
    Pool: Send + Sync,
    T: Connection<Pool> + Send + Sync,
{
    async fn is_initialized(&self, database: &T) -> bool {
        database.establish_admin_connection().await.is_ok()
    }

    async fn initialize_database(
        &self,
        database: &T,
    ) -> Result<(), <T as Connection<Pool>>::Error> {
        let (is_initialized, pool) = join(
            self.is_initialized(database),
            database.establish_default_connection(),
        )
        .await;

        match (is_initialized, pool) {
            (false, Ok(pool)) => {
                self.create_admin_database(&pool).await?;
                self.create_tenant_database_template(&pool).await?;
                database.close_connection(pool).await;
                Ok(())
            }
            (true, _) => Err(Error::AlreadyInitialized.into()),
            (false, Err(_)) => Err(Error::FailedToInitialize.into()),
        }
    }

    async fn create_admin_database(
        &self,
        pool: &Pool,
    ) -> Result<(), <T as Connection<Pool>>::Error>;

    async fn create_tenant_database_template(
        &self,
        pool: &Pool,
    ) -> Result<(), <T as Connection<Pool>>::Error>;
}

#[async_trait::async_trait]
pub trait Migrate<'a, Pool, T>: MigrateAdmin<'a, Pool, T> + MigrateTenant<'a, Pool, T>
where
    Pool: Send + Sync + IntoSchemaManagerConnection<'a>,
    T: Connection<Pool> + Send + Sync,
{
    async fn migrate_database(&self, pool: &T) -> Result<(), <T as Connection<Pool>>::Error> {
        self.run_admin_migrations(pool).await?;
        self.run_tenant_migrations(pool).await
    }
}

#[async_trait::async_trait]
pub trait MigrateAdmin<'a, Pool, T>
where
    Pool: Send + Sync + IntoSchemaManagerConnection<'a>,
    T: Connection<Pool> + Send + Sync,
{
    async fn run_admin_migrations(
        &self,
        database: &T,
    ) -> Result<(), <T as Connection<Pool>>::Error> {
        let pool = database.establish_admin_connection().await?;

        admin_migrations::Migrator::up(pool, None)
            .await
            .map_err(|e| Error::FailedToMigrate(e.into()))?;
        Ok(())
    }
}

#[async_trait::async_trait]
pub trait MigrateTenant<'a, Pool, T>
where
    Pool: Send + Sync + IntoSchemaManagerConnection<'a>,
    T: Connection<Pool> + Send + Sync,
{
    async fn run_tenant_migrations(
        &self,
        database: &T,
    ) -> Result<(), <T as Connection<Pool>>::Error> {
        let pool = database.establish_tenant_connection("tenant").await?;
        tenant_migrations::Migrator::up(pool, None)
            .await
            .map_err(|e| Error::FailedToMigrate(e.into()))?;
        Ok(())
    }
}
