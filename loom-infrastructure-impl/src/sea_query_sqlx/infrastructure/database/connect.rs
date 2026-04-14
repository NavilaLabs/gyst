use std::{str::FromStr, time::Duration};

use loom_infrastructure::{
    config::CONFIG,
    database::{
        DatabaseUri,
        database_uri_factory::{self, DatabaseUriType},
    },
};
use sqlx::any::AnyPoolOptions;
use tracing::info;

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

        let mut pool = AnyPoolOptions::new();
        let pool_config = CONFIG.get_database().get_pool();
        let max_size = pool_config.get_max_size();
        pool = pool.max_connections(max_size);
        let min_size = pool_config.get_min_size();
        pool = pool.min_connections(min_size);
        let timeout_seconds = pool_config.get_timeout_seconds();
        pool = pool.idle_timeout(Duration::from_secs(timeout_seconds));
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
            .get_uri(&DatabaseType::Sqlite.to_string(), Some(tenant_token))?;

        Self::connect(&uri).await
    }
}

impl Pool<ScopeAdmin, StateDisconnected> {
    /// # Errors
    ///
    /// Returns an error if the admin URI cannot be built or the pool cannot connect.
    pub async fn connect_admin() -> Result<Pool<ScopeAdmin, StateConnected>, Error> {
        let uri = database_uri_factory::Factory::new_database_uri(&DatabaseUriType::Admin)
            .get_uri(&DatabaseType::Sqlite.to_string(), None)?;

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
        Self::connect(
            &url::Url::from_str("sqlite:///file:loom?mode=memory&cache=shared")
                .unwrap()
                .into(),
        )
        .await
    }
}
