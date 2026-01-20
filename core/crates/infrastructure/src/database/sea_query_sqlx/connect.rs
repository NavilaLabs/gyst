use std::{marker::PhantomData, time::Duration};

use sqlx::any::AnyPoolOptions;
use tracing::info;
use url::Url;

use crate::{
    config::CONFIG,
    database::{
        Pool,
        sea_query_sqlx::{Error, StateConnected, StateDisconnected},
    },
};

impl<Scope> Pool<Scope, StateDisconnected> {
    pub async fn connect(uri: &Url) -> Result<Pool<Scope, StateConnected>, Error> {
        sqlx::any::install_default_drivers();

        let mut pool = AnyPoolOptions::new();
        if let Some(pool_config) = CONFIG.get_database().get_pool() {
            if let Some(max_size) = pool_config.get_max_size() {
                pool = pool.max_connections(max_size);
            }
            if let Some(min_size) = pool_config.get_min_size() {
                pool = pool.min_connections(min_size);
            }
            if let Some(timeout_seconds) = pool_config.get_timeout_seconds() {
                pool = pool.idle_timeout(Duration::from_secs(timeout_seconds));
            }
        }
        info!("Configured database pool: {:?}", pool);

        info!("Establishing connection to database at URL: {}", uri);
        Ok(Pool {
            pool: Some(pool.connect(uri.as_str()).await?),
            _scope: PhantomData,
            _state: PhantomData,
        })
    }
}
