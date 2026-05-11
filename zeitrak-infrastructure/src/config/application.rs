use serde::{Deserialize, Serialize};
use crate::config::{default_true, default_300};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "application")]
pub struct Application {
    environment: String,
    name: String,
    project_root: String,
    security: SecurityConfig,
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default)]
    smtp: Option<SmtpConfig>,
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
    pub fn base_url(&self) -> Option<&str> {
        self.base_url.as_deref()
    }

    /// SMTP configuration for sending emails, if configured.
    #[must_use]
    pub const fn smtp(&self) -> Option<&SmtpConfig> {
        self.smtp.as_ref()
    }
}

/// Security configuration for the application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    authentication_secret: String,
    #[serde(default = "default_true")]
    invite_only: bool,
    #[serde(default = "default_300")]
    invite_token_expiry: chrono::Duration,
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
