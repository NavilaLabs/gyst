use async_trait::async_trait;
use domain::tenant::value_objects::TenantToken;

use crate::database::{Error, Pool, ScopeDefault, StateConnected, migrate::Migrate};

#[async_trait]
impl Migrate for Pool<ScopeDefault, StateConnected> {
    type Error = Error;

    async fn migrate_admin_database(&self) -> Result<(), <Self as Migrate>::Error> {
        todo!()
    }

    async fn migrate_tenant_database(
        &self,
        _tenant_token: Option<TenantToken>,
    ) -> Result<(), <Self as Migrate>::Error> {
        todo!()
    }
}
