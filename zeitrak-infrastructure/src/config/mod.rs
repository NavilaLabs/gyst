use figment::{
    Figment,
    providers::{Env, Format, Serialized, Yaml},
};
use serde::{Deserialize, Serialize};

mod application;
mod database;

pub use application::{Application, SecurityConfig, SmtpConfig};
pub use database::{AdminDatabase, Database, Databases, Pool, TenantDatabase};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{builder}: Builder missing field | field={field}")]
    BuilderMissingField { builder: String, field: String },
}

pub static CONFIG: std::sync::LazyLock<Config> =
    std::sync::LazyLock::new(|| load_config().expect("Failed to load configuration"));

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    application: application::Application,
    database: database::Database,
}

impl Config {
    #[must_use]
    pub const fn application(&self) -> &application::Application {
        &self.application
    }

    #[must_use]
    pub const fn database(&self) -> &database::Database {
        &self.database
    }
}

/// Loads configuration from the layered sources in priority order (lowest → highest):
///
/// 1. Rust struct `Default` implementations
/// 2. `config/{environment}/application.yaml`
/// 3. `config/{environment}/database.yaml`
/// 4. Environment variables prefixed with `ZK_`, using `__` as the nested key separator
///    (e.g. `ZK_DATABASE__BASE_URI` overrides `database.base_uri`)
///
/// `ZK_ENVIRONMENT` selects the config directory (default: `development`).
/// `ZK_PROJECT_ROOT` locates the workspace root (default: `.`).
///
/// # Errors
///
/// Returns an error if the YAML content cannot be deserialized into [`Config`].
pub fn load_config() -> Result<Config, crate::Error> {
    let environment = std::env::var("ZK_ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_string());
    let project_root = std::env::var("ZK_PROJECT_ROOT")
        .unwrap_or_else(|_| ".".to_string());
    let config_dir = format!("{project_root}/config/{environment}");

    let config = Figment::new()
        .merge(Serialized::defaults(Config::default()))
        .merge(Yaml::file(format!("{config_dir}/application.yaml")))
        .merge(Yaml::file(format!("{config_dir}/database.yaml")))
        .merge(Env::prefixed("ZK_").split("__"))
        .extract()?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    use with_lifecycle::with_lifecycle;
    use zeitrak_tests::test_lifecycle;

    #[with_lifecycle(test_lifecycle)]
    #[test]
    fn test_load_config() {
        assert_eq!(CONFIG.application().environment(), "test");
        assert_eq!(CONFIG.application().name(), "Zeitrak");
        assert_eq!(
            CONFIG.database().databases().tenant().name_prefix(),
            "test_zeitrak_tenant_"
        );
    }
}
