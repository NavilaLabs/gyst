use embassy_futures::join::join;
use zeitrak_infrastructure::database::Initialize;
use zeitrak_infrastructure_impl::{
    Error, {Pool, ScopeDefault, StateConnected},
};

pub mod postgres;
pub mod sqlite;

#[allow(dead_code)]
type ConnectedDefaultPool = Pool<ScopeDefault, StateConnected>;

#[allow(dead_code)]
async fn initialize_databases(
    pool: &ConnectedDefaultPool,
    tenant_token: &str,
) -> Result<(), Error> {
    let (admin_result, tenant_result) = join(
        pool.initialize_admin_database(),
        pool.initialize_tenant_database(Some(tenant_token)),
    )
    .await;

    admin_result?;
    tenant_result?;

    Ok(())
}

pub mod test_lifecycle {
    use zeitrak_tests::test_lifecycle;
    use sqlx::any::install_default_drivers;

    #[allow(dead_code)]
    pub fn before() {
        test_lifecycle::before();
        install_default_drivers();
    }

    #[allow(dead_code)]
    pub fn after() {
        test_lifecycle::after();
    }
}
