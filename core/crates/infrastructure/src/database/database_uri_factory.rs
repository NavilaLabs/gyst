use domain::tenant::value_objects::TenantToken;
use url::Url;

use crate::{
    config::CONFIG,
    database::{
        Error, TenantDatabaseNameBuilder, TenantDatabaseNameConcreteBuilder,
        TenantDatabaseNameDirector,
    },
};

pub trait DatabaseUri {
    fn get_uri(&self, tenant_token: Option<&TenantToken>) -> Result<Url, crate::Error>;
}

pub enum DatabaseUriType {
    Admin,
    Tenant,
}

pub struct AdminDatabaseUri;

impl DatabaseUri for AdminDatabaseUri {
    fn get_uri(&self, _tenant_token: Option<&TenantToken>) -> Result<Url, crate::Error> {
        let base_uri = CONFIG.get_database().get_base_uri();
        let admin_database_name = CONFIG.get_database().get_databases().get_admin().get_name();
        let admin_uri = format!("{}/{}", base_uri, admin_database_name);

        Ok(Url::parse(&admin_uri)?)
    }
}

pub struct TenantDatabaseUri;

impl DatabaseUri for TenantDatabaseUri {
    fn get_uri(&self, tenant_token: Option<&TenantToken>) -> Result<Url, crate::Error> {
        let base_uri = CONFIG.get_database().get_base_uri();
        let tenant_token =
            tenant_token.map_or_else(|| Err(Error::NoTenantTokenProvided), |token| Ok(token))?;
        let mut database_name_builder = TenantDatabaseNameConcreteBuilder::new();
        TenantDatabaseNameDirector::construct(&mut database_name_builder, tenant_token);
        let database_name = database_name_builder.get_tenant_database_name();
        let tenant_uri = format!("{}/{}", base_uri, database_name);

        Ok(Url::parse(&tenant_uri)?)
    }
}

pub struct Factory;

impl Factory {
    pub fn new_database_uri(database_uri_type: &DatabaseUriType) -> Box<dyn DatabaseUri> {
        match database_uri_type {
            DatabaseUriType::Admin => Box::new(AdminDatabaseUri),
            DatabaseUriType::Tenant => Box::new(TenantDatabaseUri),
        }
    }
}
