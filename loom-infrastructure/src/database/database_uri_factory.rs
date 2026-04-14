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

#[cfg(test)]
mod tests {
    use super::*;

    /// Thin helper that lets us call `ensure_sqlite_extension` without CONFIG.
    struct TestUri;
    impl CreateDatabaseUri for TestUri {
        fn get_uri(
            &self,
            _database_type: &str,
            _tenant_token: Option<&str>,
        ) -> Result<DatabaseUri, crate::Error> {
            unimplemented!("not needed for these tests")
        }
    }

    fn uri(s: &str) -> DatabaseUri {
        DatabaseUri::from(Url::parse(s).unwrap())
    }

    #[test]
    fn ensure_sqlite_extension_appends_sqlite_suffix() {
        let result = TestUri
            .ensure_sqlite_extension("sqlite", uri("sqlite:///tmp/loom_admin"))
            .unwrap();
        assert!(result.to_string().ends_with(".sqlite"));
    }

    #[test]
    fn ensure_sqlite_extension_does_not_double_append() {
        let already = uri("sqlite:///tmp/loom_admin.sqlite");
        let result = TestUri
            .ensure_sqlite_extension("sqlite", already)
            .unwrap();
        let s = result.to_string();
        assert!(s.ends_with(".sqlite"));
        assert!(!s.ends_with(".sqlite.sqlite"));
    }

    #[test]
    fn ensure_sqlite_extension_does_not_modify_postgres_uri() {
        let pg = uri("postgres://localhost/loom_admin");
        let result = TestUri
            .ensure_sqlite_extension("postgres", pg)
            .unwrap();
        assert!(!result.to_string().contains(".sqlite"));
    }

    #[test]
    fn factory_returns_admin_impl_for_admin_type() {
        let factory = Factory::new_database_uri(&DatabaseUriType::Admin);
        // Just ensure it doesn't panic and returns a boxed value we can call.
        // get_uri needs CONFIG, so we only verify the factory produces something.
        let _ = factory; // type-checks as Box<dyn CreateDatabaseUri>
    }

    #[test]
    fn factory_returns_tenant_impl_for_tenant_type() {
        let factory = Factory::new_database_uri(&DatabaseUriType::Tenant);
        let _ = factory;
    }
}
