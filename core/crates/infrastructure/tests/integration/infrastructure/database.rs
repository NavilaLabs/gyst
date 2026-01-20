use domain::tenant::value_objects::TenantToken;
use infrastructure::database::{
    Error, Initialize, Pool, ScopeAdmin, ScopeDefault, ScopeTenant, StateConnected,
};
use url::Url;

async fn reset_entire_postgres_database() -> Result<(), Error> {
    let admin_pool = get_default_postgres_pool().await?;

    // 1. Terminate other connections (must be done before dropping)
    sqlx::query(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity 
         WHERE datname = 'loom_admin' AND pid <> pg_backend_pid()",
    )
    .execute(admin_pool.as_ref())
    .await?;

    // 2. Drop the main admin database
    // Note: You cannot drop the database you are currently connected to.
    // Ensure admin_pool is connected to 'postgres' or another maintenance DB.
    sqlx::query("DROP DATABASE IF EXISTS loom_admin")
        .execute(admin_pool.as_ref())
        .await?;

    // 3. Find and drop tenant databases
    let tenants: Vec<(String,)> =
        sqlx::query_as("SELECT datname::TEXT FROM pg_database WHERE datname LIKE 'loom_tenant_%'")
            .fetch_all(admin_pool.as_ref())
            .await?;

    for (tenant_name,) in tenants {
        let drop_query = format!("DROP DATABASE IF EXISTS \"{}\"", tenant_name);
        sqlx::query(&drop_query)
            .execute(admin_pool.as_ref())
            .await?;
    }

    Ok(())
}

async fn get_default_postgres_pool() -> Result<Pool<ScopeDefault, StateConnected>, Error> {
    let database_url = "postgres://postgres:postgres@postgres-test:5432/postgres";
    Pool::connect(&Url::parse(database_url).unwrap()).await
}

async fn get_admin_postgres_pool() -> Result<Pool<ScopeAdmin, StateConnected>, Error> {
    let database_url = "postgres://admin:admin@postgres-test:5432/loom_admin";
    Pool::connect(&Url::parse(database_url).unwrap()).await
}

async fn get_tenant_postgres_pool(
    tenant_token: &TenantToken,
) -> Result<Pool<ScopeTenant, StateConnected>, Error> {
    let database_url = format!(
        "postgres://tenant:tenant@postgres-test:5432/loom_tenant_{}",
        tenant_token.as_ref()
    );
    Pool::connect(&Url::parse(&database_url).unwrap()).await
}

#[tokio::test]
async fn test_connect_to_postgres_database() {
    assert!(get_default_postgres_pool().await.is_ok());
}

#[tokio::test]
async fn test_initialize_database() {
    reset_entire_postgres_database().await.unwrap();

    let default_pool = get_default_postgres_pool().await.unwrap();

    default_pool
        .initialize_admin_database()
        .await
        .expect("Failed to initialize admin database");

    default_pool
        .initialize_tenant_database(None)
        .await
        .expect("Failed to initialize tenant database");

    let tenant_token = TenantToken::new();
    default_pool
        .initialize_tenant_database(Some(&tenant_token))
        .await
        .expect("Failed to initialize tenant database with token");
}
