use std::{borrow::Cow, fmt::Display};

use domain::tenant::value_objects::TenantToken;

use crate::config::CONFIG;

#[derive(Debug, Clone)]
pub struct TenantDatabaseName<'a>(Cow<'a, str>);

impl<'a> TenantDatabaseName<'a> {
    pub fn new() -> Self {
        TenantDatabaseName(Cow::Borrowed(""))
    }
}

impl Display for TenantDatabaseName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait Builder<'a> {
    fn with_prefix(&mut self, prefix: &str);
    fn with_tenant_token(&mut self, tenant_token: &TenantToken);
    fn get_tenant_database_name(self) -> TenantDatabaseName<'a>;
}

pub struct ConcreteBuilder<'a>(TenantDatabaseName<'a>);

impl<'a> ConcreteBuilder<'a> {
    pub fn new() -> Self {
        ConcreteBuilder(TenantDatabaseName::new())
    }
}

impl<'a> Builder<'a> for ConcreteBuilder<'a> {
    fn with_prefix(&mut self, prefix: &str) {
        self.0 = TenantDatabaseName(Cow::Owned(format!("{}{}", prefix, self.0.0)));
    }

    fn with_tenant_token(&mut self, tenant_token: &TenantToken) {
        self.0 = TenantDatabaseName(Cow::Owned(format!("{}{}", self.0.0, tenant_token)));
    }

    fn get_tenant_database_name(self) -> TenantDatabaseName<'a> {
        self.0
    }
}

pub struct Director;

impl<'a> Director {
    pub fn construct<T: Builder<'a>>(builder: &mut T, tenant_token: &TenantToken) {
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
