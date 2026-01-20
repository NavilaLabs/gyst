use async_trait::async_trait;
use domain::tenant::value_objects::TenantToken;

use crate::{
    config::CONFIG,
    database::{Error, Initialize, Pool, ScopeDefault, StateConnected},
};

#[async_trait]
impl Initialize for Pool<ScopeDefault, StateConnected> {
    type Error = Error;

    async fn is_initialized(
        &self,
        tenant_token: Option<&TenantToken>,
    ) -> Result<bool, <Self as Initialize>::Error> {
        let admin_database_exists_query = "SELECT EXISTS (
            SELECT 1 
            FROM pg_database 
            WHERE datname = $1
        );";

        let database_name = match tenant_token {
            Some(token) => {
                let tenant_database_prefix = CONFIG
                    .get_database()
                    .get_databases()
                    .get_tenant()
                    .get_name_prefix();
                format!("{}_{}", tenant_database_prefix, token.as_ref())
            }
            None => CONFIG
                .get_database()
                .get_databases()
                .get_admin()
                .get_name()
                .to_string(),
        };

        let result: (bool,) = sqlx::query_as(admin_database_exists_query)
            .bind(database_name)
            .fetch_one(self.as_ref())
            .await?;

        Ok(result.0)
    }

    async fn initialize_admin_database(&self) -> Result<(), <Self as Initialize>::Error> {
        if self.is_initialized(None).await? {
            return Ok(());
        }

        let database_name = CONFIG.get_database().get_databases().get_admin().get_name();

        let query = format!(r#"CREATE DATABASE "{}""#, database_name);

        sqlx::query(query.as_str()).execute(self.as_ref()).await?;

        Ok(())
    }

    async fn initialize_tenant_database(
        &self,
        tenant_token: Option<&TenantToken>,
    ) -> Result<(), <Self as Initialize>::Error> {
        let tenant_database_prefix = CONFIG
            .get_database()
            .get_databases()
            .get_tenant()
            .get_name_prefix();
        let template_name = format!("{}_template", tenant_database_prefix);
        let query = match tenant_token {
            Some(token) => format!(
                r#"CREATE DATABASE "{}_{}" TEMPLATE "{}""#,
                tenant_database_prefix,
                token.as_ref(),
                template_name
            ),
            None => format!(r#"CREATE DATABASE "{template_name}""#),
        };

        sqlx::query(query.as_str()).execute(self.as_ref()).await?;

        Ok(())
    }
}
