use url::Url;

use crate::{
    config::CONFIG,
    database::{
        DatabaseUri, TenantDatabaseNameBuilder, TenantDatabaseNameConcreteBuilder,
        TenantDatabaseNameDirector,
    },
};

pub trait CreateDatabaseUri {
    /// # Errors
    ///
    /// Returns an error if the URI cannot be constructed or parsed.
    fn get_uri(
        &self,
        database_type: &str,
        tenant_token: Option<&str>,
    ) -> Result<DatabaseUri, crate::Error>;

    /// Ensures that the database URI has a `.sqlite` extension for `SQLite` databases.
    ///
    /// # Errors
    ///
    /// Returns an error if the modified URI cannot be parsed.
    fn ensure_sqlite_extension(
        &self,
        database_type: &str,
        database_uri: DatabaseUri,
    ) -> Result<DatabaseUri, crate::Error> {
        if database_type == "sqlite" {
            let mut uri = database_uri.to_string();
            if !uri.ends_with(".sqlite") {
                uri.push_str(".sqlite");
            }
            return Ok(Url::parse(&uri)?.into());
        }
        Ok(database_uri)
    }
}

pub enum DatabaseUriType {
    Admin,
    Tenant,
}

pub struct AdminDatabaseUri;

impl CreateDatabaseUri for AdminDatabaseUri {
    fn get_uri(
        &self,
        database_type: &str,
        _tenant_token: Option<&str>,
    ) -> Result<DatabaseUri, crate::Error> {
        let base_uri = CONFIG.get_database().get_base_uri();
        let admin_database_name = CONFIG.get_database().get_databases().get_admin().get_name();
        let admin_uri = Url::parse(&format!("{base_uri}/{admin_database_name}"))?;
        let admin_uri = self.ensure_sqlite_extension(database_type, admin_uri.into())?;

        Ok(admin_uri)
    }
}

pub struct TenantDatabaseUri;

impl CreateDatabaseUri for TenantDatabaseUri {
    fn get_uri(
        &self,
        database_type: &str,
        tenant_token: Option<&str>,
    ) -> Result<DatabaseUri, crate::Error> {
        let base_uri = CONFIG.get_database().get_base_uri();
        let tenant_token =
            tenant_token.map_or_else(|| Err(crate::database::Error::NoTenantTokenProvided), Ok)?;
        let mut database_name_builder = TenantDatabaseNameConcreteBuilder::new();
        TenantDatabaseNameDirector::construct(&mut database_name_builder, tenant_token);
        let database_name = database_name_builder.get_tenant_database_name();
        let tenant_uri = Url::parse(&format!("{base_uri}/{database_name}"))?;
        let tenant_uri = self.ensure_sqlite_extension(database_type, tenant_uri.into())?;

        Ok(tenant_uri)
    }
}

pub struct Factory;

impl Factory {
    #[must_use]
    pub fn new_database_uri(database_uri_type: &DatabaseUriType) -> Box<dyn CreateDatabaseUri> {
        match database_uri_type {
            DatabaseUriType::Admin => Box::new(AdminDatabaseUri),
            DatabaseUriType::Tenant => Box::new(TenantDatabaseUri),
        }
    }
}
