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
            .map(|s| {
                // Strip any file extension (e.g. ".sqlite") before parsing.
                s.split('.').next().unwrap_or(s)
            })
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> DatabaseUri {
        DatabaseUri::from(Url::parse(s).expect("valid URL"))
    }

    #[test]
    fn from_url_preserves_the_url() {
        let url = Url::parse("sqlite:///tmp/zeitrak_admin.sqlite").unwrap();
        let uri = DatabaseUri::from(url.clone());
        assert_eq!(uri.as_ref(), &url);
    }

    #[test]
    fn display_shows_the_url_string() {
        let s = "sqlite:///tmp/zeitrak_admin.sqlite";
        let uri = parse(s);
        assert_eq!(uri.to_string(), s);
    }

    #[test]
    fn deref_gives_access_to_url_scheme() {
        let uri = parse("sqlite:///tmp/zeitrak.sqlite");
        assert_eq!(uri.scheme(), "sqlite");
    }

    #[test]
    fn tenant_token_is_none_for_plain_admin_path() {
        let uri = parse("sqlite:///tmp/zeitrak_admin.sqlite");
        assert!(uri.tenant_token().is_none());
    }

    #[test]
    fn tenant_token_is_extracted_when_path_ends_with_valid_uuid() {
        // The path segment after the last `_` must be a valid UUID.
        let uuid_str = "019d0ce8-facb-7c90-b9d7-287ae4f17c91";
        let uri = parse(&format!("sqlite:///tmp/zeitrak_tenant_{uuid_str}.sqlite"));
        let token = uri.tenant_token();
        assert!(token.is_some(), "expected a tenant token to be extracted");
        assert_eq!(token.unwrap().to_string(), uuid_str);
    }

    #[test]
    fn tenant_token_is_none_when_suffix_is_not_a_uuid() {
        let uri = parse("sqlite:///tmp/zeitrak_tenant_not_a_uuid.sqlite");
        assert!(uri.tenant_token().is_none());
    }
}
