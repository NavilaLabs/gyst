use async_trait::async_trait;
use domain::tenant::value_objects::TenantToken;

#[async_trait]
pub trait Migrate {
    type Error;

    async fn migrate_admin_database(&self) -> Result<(), <Self as Migrate>::Error>;

    async fn migrate_tenant_database(
        &self,
        tenant_token: Option<TenantToken>,
    ) -> Result<(), <Self as Migrate>::Error>;
}
