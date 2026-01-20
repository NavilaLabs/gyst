use async_trait::async_trait;
use domain::tenant::value_objects::TenantToken;

#[async_trait]
pub trait Initialize {
    type Error;

    async fn is_initialized(&self, tenant_token: Option<&TenantToken>)
    -> Result<bool, Self::Error>;

    async fn initialize_admin_database(&self) -> Result<(), Self::Error>;

    async fn initialize_tenant_database(
        &self,
        tenant_token: Option<&TenantToken>,
    ) -> Result<(), Self::Error>;
}
