use std::time::Duration;

use crate::{config, database};
use log::info;
use sqlx::postgres::PgPoolOptions;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::Error),
    #[error("Database error: {0}")]
    DatabaseError(#[from] database::Error),
    #[error("SQLx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
}

#[derive(Debug)]
pub struct Pool;

#[derive(Debug)]
pub struct Database;

#[async_trait::async_trait]
impl database::Connection<sqlx::PgPool> for Pool {
    #[cfg(feature = "sea-query-sqlx-postgres")]
    const DEFAULT_DATABASE_NAME: &'static str = "postgres";
    #[cfg(feature = "sea-query-sqlx-sqlite")]
    const DEFAULT_DATABASE_NAME: &'static str = "???";
    type Error = Error;

    async fn establish_connection(&self, database_url: &str) -> Result<sqlx::PgPool, Self::Error> {
        let database_config = config::CONFIG.get_database();
        let url = database_config.get_postgres_uri(database_url)?;
        info!("Establishing connection to database at URL: {}", url);

        let mut pool = PgPoolOptions::new();
        if let Some(pool_config) = database_config.get_pool() {
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

        Ok(pool.connect(url.as_str()).await?)
    }

    async fn close_connection(&self, pool: sqlx::PgPool) {
        pool.close().await;
    }
}

#[async_trait::async_trait]
impl database::Initialize<sqlx::PgPool, Pool> for Database {
    async fn create_admin_database(
        &self,
        pool: &sqlx::PgPool,
    ) -> Result<(), <Pool as database::Connection<sqlx::PgPool>>::Error> {
        let database_name = config::CONFIG
            .get_database()
            .get_databases()
            .get_admin()
            .get_name();

        let query = format!(r#"CREATE DATABASE "{}""#, database_name);
        sqlx::query(&query).execute(pool).await?;

        Ok(())
    }

    async fn create_tenant_database_template(
        &self,
        pool: &sqlx::PgPool,
    ) -> Result<(), <Pool as database::Connection<sqlx::PgPool>>::Error> {
        let template_name = config::CONFIG
            .get_database()
            .get_databases()
            .get_tenant()
            .get_name_prefix();

        let query = format!(r#"CREATE DATABASE "{}_template""#, template_name);
        sqlx::query(&query).execute(pool).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use infrastructure::database::{Connection, Initialize};

    #[tokio::test]
    async fn test_initialize_database() {
        let db = super::Database;

        // db.initialize_database(&db).await.unwrap();
        todo!()
    }

    #[tokio::test]
    async fn test_establish_admin_connection() {
        let db = super::Database;

        db.establish_admin_connection().await.unwrap();
    }
}
