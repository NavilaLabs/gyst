use domain::tenant::value_objects::TenantToken;
use infrastructure::{
    Error,
    config::CONFIG,
    database::{
        Migrate, Pool, ScopeAdmin, ScopeTenant, StateConnected, TenantDatabaseNameBuilder,
        TenantDatabaseNameConcreteBuilder, TenantDatabaseNameDirector,
    },
};
use tracing::info;
use url::Url;

use super::{ConnectedDefaultPool, initialize_databases};

async fn reset_entire_database(pool: &ConnectedDefaultPool) -> Result<(), Error> {
    // 1. Terminate other connections (must be done before dropping)
    sqlx::query(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity
         WHERE datname LIKE 'test_loom_%' AND pid <> pg_backend_pid()",
    )
    .execute(pool.as_ref())
    .await?;

    // 2. Drop the main admin database
    let admin_database_name = CONFIG.get_database().get_databases().get_admin().get_name();
    sqlx::query(&format!(
        "DROP DATABASE IF EXISTS \"{admin_database_name}\""
    ))
    .execute(pool.as_ref())
    .await?;

    // 3. Find and drop tenant databases
    let tenant_database_name_prefix = CONFIG
        .get_database()
        .get_databases()
        .get_tenant()
        .get_name_prefix();
    let tenants: Vec<(String,)> = sqlx::query_as(&format!(
        "SELECT datname::TEXT FROM pg_database WHERE datname LIKE '{tenant_database_name_prefix}_%'"
    ))
    .fetch_all(pool.as_ref())
    .await?;

    for (tenant_name,) in tenants {
        let drop_query = format!("DROP DATABASE IF EXISTS \"{tenant_name}\"");
        sqlx::query(&drop_query).execute(pool.as_ref()).await?;
    }

    Ok(())
}

async fn get_default_pool() -> Result<ConnectedDefaultPool, Error> {
    let database_url = "postgres://postgres:postgres@postgres-test:5432/postgres";
    Pool::connect(&Url::parse(database_url).unwrap()).await
}

async fn get_admin_pool() -> Result<Pool<ScopeAdmin, StateConnected>, Error> {
    let admin_database_name = CONFIG.get_database().get_databases().get_admin().get_name();
    let database_url = format!(
        "postgres://postgres:postgres@postgres-test:5432/{}",
        admin_database_name
    );
    Pool::connect(&Url::parse(&database_url).unwrap()).await
}

async fn get_tenant_pool(
    tenant_token: &TenantToken,
) -> Result<Pool<ScopeTenant, StateConnected>, Error> {
    let mut database_name_builder = TenantDatabaseNameConcreteBuilder::new();
    TenantDatabaseNameDirector::construct(&mut database_name_builder, tenant_token);
    let database_name = database_name_builder.get_tenant_database_name();
    let database_url = format!("postgres://postgres:postgres@postgres-test:5432/{database_name}",);
    Pool::connect(&Url::parse(&database_url).unwrap()).await
}

pub(crate) async fn refresh_databases(
    pool: &ConnectedDefaultPool,
    tenant_token: &TenantToken,
) -> Result<(), Error> {
    reset_entire_database(pool).await?;
    info!("Database successfully reseted!");
    initialize_databases(pool, tenant_token).await?;
    info!("Database successfully initialized!");

    let admin_pool = get_admin_pool().await?;
    admin_pool.migrate_database().await?;
    let tenant_template_pool = get_tenant_pool(tenant_token).await?;
    tenant_template_pool.migrate_database().await?;
    info!("Database successfully refreshed!");

    Ok(())
}

pub mod tests {
    use with_lifecycle::with_lifecycle;

    use crate::database::test_lifecycle;

    use super::*;

    #[with_lifecycle(test_lifecycle)]
    #[tokio::test]
    async fn test_setup_postgres_database() -> Result<(), Error> {
        let default_pool = get_default_pool().await?;
        refresh_databases(&default_pool, &TenantToken::default()).await?;

        Ok(())
    }
}
