use async_trait::async_trait;

#[async_trait]
pub trait Initialize {
    type Error;

    async fn is_initialized(&self, tenant_token: Option<&str>) -> Result<bool, Self::Error>;

    async fn initialize_admin_database(&self) -> Result<(), Self::Error>;

    async fn initialize_tenant_database(
        &self,
        tenant_token: Option<&str>,
    ) -> Result<(), Self::Error>;
}
