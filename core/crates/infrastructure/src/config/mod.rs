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
    pub fn get_application(&self) -> &application::Application {
        &self.application
    }

    #[must_use]
    pub fn get_database(&self) -> &database::Database {
        &self.database
    }
}

pub fn load_config() -> Result<Config, crate::Error> {
    let project_root = var("APP_PROJECT_ROOT")?;
    let environment = var("ENVIRONMENT")?;
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

    let mut source = serde_vars::EnvSource::default();
    let de = serde_yaml::Deserializer::from_str(&file_string);
    let config: Config = serde_vars::deserialize(de, &mut source)?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    use shared::test_lifecycle;
    use with_lifecycle::with_lifecycle;

    #[with_lifecycle(test_lifecycle)]
    #[test]
    fn test_load_config() {
        assert_eq!(CONFIG.get_application().get_environment(), "test");
        assert_eq!(CONFIG.get_application().get_name(), "Loom");
        assert_eq!(
            CONFIG
                .get_database()
                .get_databases()
                .get_tenant()
                .get_name_prefix(),
            "test_loom_tenant_"
        );
    }
}
