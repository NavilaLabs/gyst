use async_trait::async_trait;

#[async_trait]
pub trait Initialize<Pool>
where
    Pool: Send + Sync,
{
    type Error;

    async fn is_initialized<T>(&self, database: &T) -> bool
    where
        T: super::Connection<Pool> + Send + Sync,
    {
        database.establish_admin_connection().await.is_ok()
    }

    async fn initialize_admin_database(&self, pool: &Pool) -> Result<(), Self::Error>;

    async fn initialize_tenant_database(
        &self,
        pool: &Pool,
        tenant_token: Option<&str>,
    ) -> Result<(), Self::Error>;
}
