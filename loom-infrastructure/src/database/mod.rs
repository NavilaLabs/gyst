mod initialize;
use std::{fmt::Display, ops::Deref};

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

impl From<Url> for DatabaseUri {
    fn from(uri: Url) -> Self {
        let tenant_token: Option<Uuid> = uri
            .as_str()
            .split('_')
            .next_back()
            .map(Uuid::parse_str)
            .transpose()
            .ok()
            .flatten();

        Self { uri, tenant_token }
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

impl Display for DatabaseUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uri)
    }
}
