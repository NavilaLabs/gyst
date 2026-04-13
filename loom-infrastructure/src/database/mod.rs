mod initialize;
use std::ops::Deref;

pub use initialize::*;
pub mod migrate;
pub use migrate::*;
mod tenant_database_name_builder;
pub use tenant_database_name_builder::{
    Builder as TenantDatabaseNameBuilder, ConcreteBuilder as TenantDatabaseNameConcreteBuilder,
    Director as TenantDatabaseNameDirector,
};
use url::Url;
use uuid::Uuid;
pub mod database_uri_factory;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No tenant token provided")]
    NoTenantTokenProvided,
}

#[derive(Debug)]
pub struct DatabaseUri {
    uri: Url,
    tenant_token: Option<Uuid>,
}

impl DatabaseUri {
    #[must_use]
    pub const fn tenant_token(&self) -> Option<&Uuid> {
        self.tenant_token.as_ref()
    }
}

impl AsRef<Url> for DatabaseUri {
    fn as_ref(&self) -> &Url {
        &self.uri
    }
}

impl Deref for DatabaseUri {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.uri
    }
}
