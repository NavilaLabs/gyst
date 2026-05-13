use std::{str::FromStr, time::Duration};

use sqlx::any::AnyPoolOptions;
use tracing::info;
use zeitrak_infrastructure::{
    config::CONFIG,
    database::{
        DatabaseUri,
        database_uri_factory::{self, DatabaseUriType},
    },
};

use crate::{
    Error, ScopeAdmin, ScopeDefault, ScopeTenant,
    sea_query_sqlx::infrastructure::{DatabaseType, Pool, StateConnected, StateDisconnected},
};

impl<Scope> Pool<Scope, StateDisconnected> {
    /// # Errors
    ///
    /// Returns an error if the pool cannot connect to the database at `uri`.
    pub async fn connect(uri: &DatabaseUri) -> Result<Pool<Scope, StateConnected>, Error> {
        sqlx::any::install_default_drivers();
        let pool_config = CONFIG.database().pool();
        let pool = AnyPoolOptions::new()
            .max_connections(pool_config.max_size())
            .min_connections(pool_config.min_size())
            .idle_timeout(Duration::from_secs(pool_config.timeout_seconds()));
        let database_type = match uri.scheme() {
            "postgres" => DatabaseType::Postgres,
            "sqlite" => DatabaseType::Sqlite,
            schema => {
                return Err(
                    crate::sea_query_sqlx::infrastructure::Error::UnsupportedDatabaseType(
                        schema.to_string(),
                    )
                    .into(),
                );
            }
        };

        info!("Configured database pool: {:?}", pool);
        info!("Establishing connection to database at URL: {}", uri);
        let pool = Pool::new(
            StateConnected::new(pool.connect(uri.as_str()).await?),
            database_type,
            uri.tenant_token().copied(),
        );

        info!("Connected to database at URL: {uri}");

        Ok(pool)
    }
}

impl Pool<ScopeTenant, StateDisconnected> {
    /// # Errors
    ///
    /// Returns an error if the tenant URI cannot be built or the pool cannot connect.
    pub async fn connect_tenant(
        tenant_token: &str,
    ) -> Result<Pool<ScopeTenant, StateConnected>, Error> {
        let uri = database_uri_factory::Factory::new_database_uri(&DatabaseUriType::Tenant)
            .uri(&DatabaseType::Sqlite.to_string(), Some(tenant_token))?;

        Self::connect(&uri).await
    }
}

impl Pool<ScopeAdmin, StateDisconnected> {
    /// # Errors
    ///
    /// Returns an error if the admin URI cannot be built or the pool cannot connect.
    pub async fn connect_admin() -> Result<Pool<ScopeAdmin, StateConnected>, Error> {
        let uri = database_uri_factory::Factory::new_database_uri(&DatabaseUriType::Admin)
            .uri(&DatabaseType::Sqlite.to_string(), None)?;

        Self::connect(&uri).await
    }
}

impl Pool<ScopeDefault, StateDisconnected> {
    /// # Errors
    ///
    /// Returns an error if the pool cannot connect to the default in-memory database.
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded default URI fails to parse (should never happen).
    pub async fn connect_default() -> Result<Pool<ScopeDefault, StateConnected>, Error> {
        let uri = &url::Url::from_str("sqlite:///file:zeitrak?mode=memory&cache=shared")
            .expect("hardcoded default URI must parse")
            .into();

        Self::connect(uri).await
    }
}
