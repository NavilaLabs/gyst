use async_trait::async_trait;

#[async_trait]
pub trait Migrate<Pool>
where
    Pool: Send + Sync,
{
    type Error;

    async fn migrate_admin_database(
        &self,
        pool: &Pool,
    ) -> Result<(), <Self as Migrate<Pool>>::Error>;

    async fn migrate_tenant_database(
        &self,
        pool: &Pool,
    ) -> Result<(), <Self as Migrate<Pool>>::Error>;
}
