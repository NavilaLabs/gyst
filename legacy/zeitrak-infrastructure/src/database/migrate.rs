use async_trait::async_trait;

#[async_trait]
pub trait Migrate {
    type Error;

    async fn migrate_database(&self) -> Result<(), <Self as Migrate>::Error>;
}
