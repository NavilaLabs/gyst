use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "application")]
pub struct Application {
    environment: String,
    name: String,
    project_root: String,
    authentication_secret: String,
}

impl Application {
    pub fn environment(&self) -> &str {
        &self.environment
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn project_root(&self) -> &str {
        &self.project_root
    }

    pub fn authentication_secret(&self) -> &str {
        &self.authentication_secret
    }
}
