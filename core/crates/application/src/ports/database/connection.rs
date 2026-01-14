use async_trait::async_trait;

#[async_trait]
pub trait Connection<Pool>
where
    Pool: Send + Sync,
{
    type Error;

    async fn establish_admin_connection(&self) -> Result<Pool, Self::Error>;

    async fn establish_tenant_connection(&self, tenant_token: &str) -> Result<Pool, Self::Error>;

    async fn close_connection(&self, pool: Pool);
}
