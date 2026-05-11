use std::sync::LazyLock;

use dotenvy::var;
use serde::{Deserialize, Serialize};

mod application;
mod database;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{builder}: Builder missing field | field={field}")]
    BuilderMissingField { builder: String, field: String },
}

pub static CONFIG: LazyLock<Config> =
    LazyLock::new(|| load_config().expect("Failed to load configuration"));

#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// # Errors
///
/// Returns an error if required environment variables are missing, config files cannot be read,
/// or the YAML content cannot be deserialized.
pub fn load_config() -> Result<Config, crate::Error> {
    dotenvy::dotenv().ok();

    let project_root = var("ZK_PROJECT_ROOT")?;
    let environment = var("ZK_ENVIRONMENT")?;
    let config_path = format!("{project_root}/config/{environment}");

    let mut file_string = String::new();
    let application_config_path = format!("{config_path}/application.yaml");
    let database_config_path = format!("{config_path}/database.yaml");
    let logging_config_path = format!("{config_path}/logging.yaml");
    file_string.push_str(&std::fs::read_to_string(&application_config_path)?);
    file_string.push('\n');
    file_string.push_str(&std::fs::read_to_string(&database_config_path)?);
    file_string.push('\n');
    file_string.push_str(&std::fs::read_to_string(&logging_config_path)?);

    let config = config::Config::builder()
        .add_source(config::File::from_str(&file_string, config::FileFormat::Yaml))
        .add_source(config::Environment::with_prefix("DATABASE"))
        .add_source(config::Environment::with_prefix("ADMIN"))
        .build()?;
    let config: crate::config::Config = config.try_deserialize()?;

    Ok(config)
}

const fn default_true() -> bool {
    true
}

const fn default_300() -> chrono::Duration {
    chrono::Duration::seconds(300)
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
