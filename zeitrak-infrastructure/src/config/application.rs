use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "application")]
pub struct Application {
    environment: String,
    name: String,
    project_root: String,
    security: SecurityConfig,
    base_url: String,
    smtp: SmtpConfig,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            environment: "development".to_string(),
            name: "Zeitrak".to_string(),
            project_root: "/workspaces/zeitrak".to_string(),
            security: SecurityConfig::default(),
            base_url: "http://localhost:8080".to_string(),
            smtp: SmtpConfig::default(),
        }
    }
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

    pub const fn security(&self) -> &SecurityConfig {
        &self.security
    }

    /// The public base URL of the application, used to build invitation links.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// SMTP configuration for sending emails, if configured.
    #[must_use]
    pub const fn smtp(&self) -> &SmtpConfig {
        &self.smtp
    }
}

/// Security configuration for the application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    authentication_secret: String,
    invite_only: bool,
    invite_token_expiry: chrono::Duration,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            authentication_secret: String::new(),
            invite_only: true,
            invite_token_expiry: chrono::Duration::seconds(300),
        }
    }
}

impl SecurityConfig {
    pub fn authentication_secret(&self) -> &str {
        &self.authentication_secret
    }

    pub fn invite_only(&self) -> bool {
        self.invite_only
    }

    pub fn invite_token_expiry(&self) -> &chrono::Duration {
        &self.invite_token_expiry
    }
}

/// Optional SMTP configuration for sending transactional emails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    from_address: String,
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            host: "http://localhost".to_string(),
            port: 1025,
            username: "".to_string(),
            password: "".to_string(),
            from_address: "Zeitrak".to_string(),
        }
    }
}

impl SmtpConfig {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_address(&self) -> &str {
        &self.from_address
    }
}
