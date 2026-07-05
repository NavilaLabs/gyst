use std::fmt::Display;

use crate::config::CONFIG;

#[derive(Debug, Clone)]
pub struct TenantDatabaseName(String);

impl TenantDatabaseName {
    pub const fn new() -> Self {
        Self(String::new())
    }
}

impl Display for TenantDatabaseName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait Builder {
    fn with_prefix(&mut self, prefix: &str);
    fn with_tenant_token(&mut self, tenant_token: &str);
    fn tenant_database_name(self) -> TenantDatabaseName;
}

pub struct ConcreteBuilder(TenantDatabaseName);

impl Default for ConcreteBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ConcreteBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self(TenantDatabaseName::new())
    }
}

impl Builder for ConcreteBuilder {
    fn with_prefix(&mut self, prefix: &str) {
        self.0 = TenantDatabaseName(format!("{}{}", prefix, self.0.0));
    }

    fn with_tenant_token(&mut self, tenant_token: &str) {
        self.0 = TenantDatabaseName(format!("{}{}", self.0.0, tenant_token));
    }

    fn tenant_database_name(self) -> TenantDatabaseName {
        self.0
    }
}

pub struct Director;

impl Director {
    pub fn construct<T: Builder>(builder: &mut T, tenant_token: &str) {
        builder.with_prefix(CONFIG.database().databases().tenant().name_prefix());
        builder.with_tenant_token(tenant_token);
    }
}
