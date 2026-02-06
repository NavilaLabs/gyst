use async_trait::async_trait;
use domain::tenant::value_objects::TenantToken;
use embassy_futures::join::join;
use sea_query::{Expr, ExprTrait, PostgresQueryBuilder, Query};
use sea_query_sqlx::SqlxBinder;
use sqlx::Row;
use tracing::info;
use url::Url;

use crate::{
    Error,
    config::CONFIG,
    database::{
        self, DatabaseUriFactory, DatabaseUriType, Initialize, Pool, TenantDatabaseNameBuilder,
        TenantDatabaseNameConcreteBuilder, TenantDatabaseNameDirector, database_uri_factory,
        sea_query_sqlx::{DatabaseType, ScopeDefault, StateConnected},
    },
};

#[async_trait]
impl Initialize for Pool<ScopeDefault, StateConnected> {
    type Error = Error;

    async fn is_initialized(
        &self,
        tenant_token: Option<&TenantToken>,
    ) -> Result<bool, <Self as Initialize>::Error> {
        match self.database_type {
            DatabaseType::Postgres => {
                Initializer::new(PostgresInitializationStrategy)
                    .is_initialized(&self, tenant_token)
                    .await
            }
            DatabaseType::Sqlite => {
                Initializer::new(SqliteInitializationStrategy)
                    .is_initialized(&self, tenant_token)
                    .await
            }
        }
    }

    async fn initialize_admin_database(&self) -> Result<(), <Self as Initialize>::Error> {
        match self.database_type {
            DatabaseType::Postgres => {
                Initializer::new(PostgresInitializationStrategy)
                    .initialize_admin(&self)
                    .await
            }
            DatabaseType::Sqlite => {
                Initializer::new(SqliteInitializationStrategy)
                    .initialize_admin(&self)
                    .await
            }
        }
    }

    async fn initialize_tenant_database(
        &self,
        tenant_token: Option<&TenantToken>,
    ) -> Result<(), <Self as Initialize>::Error> {
        match self.database_type {
            DatabaseType::Postgres => {
                Initializer::new(PostgresInitializationStrategy)
                    .initialize_tenant(&self, tenant_token)
                    .await
            }
            DatabaseType::Sqlite => {
                Initializer::new(SqliteInitializationStrategy)
                    .initialize_tenant(&self, tenant_token)
                    .await
            }
        }
    }
}

#[async_trait]
pub trait InitializationStrategy {
    async fn check_is_initialized(
        &self,
        pool: &Pool<ScopeDefault, StateConnected>,
        database_uri: &Url,
    ) -> Result<bool, Error>;

    async fn is_admin_initialized(
        &self,
        pool: &Pool<ScopeDefault, StateConnected>,
    ) -> Result<bool, Error> {
        let admin_database_uri = database_uri_factory::Factory::new_database_uri(
            &crate::database::DatabaseUriType::Admin,
        )
        .get_uri(None)?;

        self.check_is_initialized(pool, &admin_database_uri).await
    }

    async fn is_tenant_initialized(
        &self,
        pool: &Pool<ScopeDefault, StateConnected>,
        tenant_token: Option<&TenantToken>,
    ) -> Result<bool, Error> {
        let database_uri = database_uri_factory::Factory::new_database_uri(
            &crate::database::DatabaseUriType::Tenant,
        )
        .get_uri(tenant_token)?;
        dbg!(&database_uri);

        self.check_is_initialized(pool, &database_uri).await
    }

    async fn is_initialized(
        &self,
        pool: &Pool<ScopeDefault, StateConnected>,
        tenant_token: Option<&TenantToken>,
    ) -> Result<bool, Error> {
        let admin_fut = self.is_admin_initialized(pool);
        let tenant_fut = self.is_tenant_initialized(pool, tenant_token);
        let (admin_result, tenant_result): (Result<bool, Error>, Result<bool, Error>) =
            join(admin_fut, tenant_fut).await;

        match (admin_result, tenant_result) {
            (Ok(admin), Ok(tenant)) => Ok(admin && tenant),
            (Err(err), _) | (_, Err(err)) => Err(err),
        }
    }

    async fn initialize_admin(
        &self,
        pool: &Pool<ScopeDefault, StateConnected>,
    ) -> Result<(), Error>;

    async fn initialize_tenant(
        &self,
        pool: &Pool<ScopeDefault, StateConnected>,
        tenant_token: Option<&TenantToken>,
    ) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct PostgresInitializationStrategy;

#[async_trait]
impl InitializationStrategy for PostgresInitializationStrategy {
    async fn check_is_initialized(
        &self,
        pool: &Pool<ScopeDefault, StateConnected>,
        database_uri: &Url,
    ) -> Result<bool, Error> {
        let (sql, _) = Query::select()
            .expr(Expr::exists(
                Query::select()
                    .expr(Expr::value(1))
                    .from("pg_database")
                    .and_where(Expr::col("datname").eq(database_uri.host_str().unwrap_or("")))
                    .to_owned(),
            ))
            .build_sqlx(PostgresQueryBuilder);
        let result = sqlx::query(&sql).fetch_one(pool.as_ref()).await?;

        Ok(result.get(0))
    }

    async fn initialize_admin(
        &self,
        pool: &Pool<ScopeDefault, StateConnected>,
    ) -> Result<(), Error> {
        let database_name = CONFIG.get_database().get_databases().get_admin().get_name();
        info!("Initializing admin database: {}", database_name);
        let query = format!(r#"CREATE DATABASE "{}""#, database_name);
        sqlx::query(query.as_str()).execute(pool.as_ref()).await?;

        Ok(())
    }

    async fn initialize_tenant(
        &self,
        pool: &Pool<ScopeDefault, StateConnected>,
        tenant_token: Option<&TenantToken>,
    ) -> Result<(), Error> {
        let tenant_token = tenant_token.map_or_else(
            || Err(database::Error::NoTenantTokenProvided),
            |token| Ok(token),
        )?;
        let mut database_name_builder = TenantDatabaseNameConcreteBuilder::new();
        TenantDatabaseNameDirector::construct(&mut database_name_builder, tenant_token);
        let database_name = database_name_builder.get_tenant_database_name();

        info!("Initializing tenant database: {}", database_name);
        let query = format!(r#"CREATE DATABASE "{}""#, database_name);
        sqlx::query(query.as_str()).execute(pool.as_ref()).await?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct SqliteInitializationStrategy;

#[async_trait]
impl InitializationStrategy for SqliteInitializationStrategy {
    async fn check_is_initialized(
        &self,
        _pool: &Pool<ScopeDefault, StateConnected>,
        database_uri: &Url,
    ) -> Result<bool, Error> {
        let file_path = format!("{}.sqlite", database_uri.path());

        Ok(std::fs::metadata(&file_path).is_ok())
    }

    async fn initialize_admin(
        &self,
        _pool: &Pool<ScopeDefault, StateConnected>,
    ) -> Result<(), Error> {
        let uri = DatabaseUriFactory::new_database_uri(&DatabaseUriType::Admin).get_uri(None)?;
        let uri = uri.to_string().replace("sqlite://", "");
        info!("Initializing admin database: {}", uri);
        std::fs::File::create(&uri)?;

        Ok(())
    }

    async fn initialize_tenant(
        &self,
        _pool: &Pool<ScopeDefault, StateConnected>,
        tenant_token: Option<&TenantToken>,
    ) -> Result<(), Error> {
        let tenant_token = tenant_token.map_or_else(
            || Err(database::Error::NoTenantTokenProvided),
            |token| Ok(token),
        )?;
        let uri = DatabaseUriFactory::new_database_uri(&DatabaseUriType::Tenant)
            .get_uri(Some(tenant_token))?;
        let uri = uri.to_string().replace("sqlite://", "");
        info!("Initializing tenant database: {}", uri);
        std::fs::File::create(&uri)?;

        Ok(())
    }
}

pub struct Initializer<T> {
    strategy: T,
}

impl<T: InitializationStrategy + Send + Sync> Initializer<T> {
    pub fn new(strategy: T) -> Self {
        Self { strategy }
    }

    pub async fn is_initialized(
        &self,
        pool: &Pool<ScopeDefault, StateConnected>,
        tenant_token: Option<&TenantToken>,
    ) -> Result<bool, Error> {
        self.strategy.is_initialized(pool, tenant_token).await
    }

    pub async fn initialize_admin(
        &self,
        pool: &Pool<ScopeDefault, StateConnected>,
    ) -> Result<(), Error> {
        self.strategy.initialize_admin(pool).await
    }

    pub async fn initialize_tenant(
        &self,
        pool: &Pool<ScopeDefault, StateConnected>,
        tenant_token: Option<&TenantToken>,
    ) -> Result<(), Error> {
        self.strategy.initialize_tenant(pool, tenant_token).await
    }
}
