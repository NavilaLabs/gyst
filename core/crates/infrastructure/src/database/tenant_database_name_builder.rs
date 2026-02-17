use std::fmt::Display;

use domain::tenant::value_objects::TenantToken;

use crate::config::CONFIG;

#[derive(Debug, Clone)]
pub struct TenantDatabaseName(String);

impl TenantDatabaseName {
    pub fn new() -> Self {
        TenantDatabaseName(String::new())
    }
}

impl Display for TenantDatabaseName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait Builder {
    fn with_prefix(&mut self, prefix: &str);
    fn with_tenant_token(&mut self, tenant_token: &TenantToken);
    fn get_tenant_database_name(self) -> TenantDatabaseName;
}

pub struct ConcreteBuilder(TenantDatabaseName);

impl ConcreteBuilder {
    pub fn new() -> Self {
        ConcreteBuilder(TenantDatabaseName::new())
    }
}

impl Builder for ConcreteBuilder {
    fn with_prefix(&mut self, prefix: &str) {
        self.0 = TenantDatabaseName(format!("{}{}", prefix, self.0.0));
    }

    fn with_tenant_token(&mut self, tenant_token: &TenantToken) {
        self.0 = TenantDatabaseName(format!("{}{}", self.0.0, tenant_token));
    }

    fn get_tenant_database_name(self) -> TenantDatabaseName {
        self.0
    }
}

pub struct Director;

impl Director {
    pub fn construct<T: Builder>(builder: &mut T, tenant_token: &TenantToken) {
        builder.with_prefix(
            CONFIG
                .get_database()
                .get_databases()
                .get_tenant()
                .get_name_prefix(),
        );
        builder.with_tenant_token(tenant_token);
    }
}
