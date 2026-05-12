use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// The active environment name (e.g. `development`, `production`).
    #[must_use]
    pub fn environment(&self) -> &str {
        &self.environment
    }

    /// The human-readable application name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Absolute path to the project workspace root.
    #[must_use]
    pub fn project_root(&self) -> &str {
        &self.project_root
    }

    /// Security-related settings (JWT secret, invite policy, etc.).
    #[must_use]
    pub const fn security(&self) -> &SecurityConfig {
        &self.security
    }

    /// The public base URL of the application, used to build invitation links.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// SMTP configuration for sending transactional emails.
    #[must_use]
    pub const fn smtp(&self) -> &SmtpConfig {
        &self.smtp
    }
}

/// Security configuration (authentication, invitation policy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    authentication_secret: String,
    invite_only: bool,
    /// How long an invitation token stays valid, in seconds.
    invite_token_expiry_secs: i64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            authentication_secret: String::new(),
            invite_only: true,
            invite_token_expiry_secs: 300,
        }
    }
}

impl SecurityConfig {
    /// The HS256 secret used to sign and verify JWT tokens.
    #[must_use]
    pub fn authentication_secret(&self) -> &str {
        &self.authentication_secret
    }

    /// Whether registration is restricted to invited users only.
    #[must_use]
    pub const fn invite_only(&self) -> bool {
        self.invite_only
    }

    /// How long an invitation token stays valid.
    #[must_use]
    pub const fn invite_token_expiry(&self) -> chrono::Duration {
        chrono::Duration::seconds(self.invite_token_expiry_secs)
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
    /// When `false`, connect over plain SMTP with no TLS (suitable for
    /// local dev tools such as MailHog).  Defaults to `true`.
    #[serde(default = "SmtpConfig::default_use_tls")]
    use_tls: bool,
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 1025,
            username: String::new(),
            password: String::new(),
            from_address: "noreply@zeitrak.app".to_string(),
            use_tls: true,
        }
    }
}

impl SmtpConfig {
    fn default_use_tls() -> bool {
        true
    }

    /// SMTP server hostname or IP address.
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// SMTP server port.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// SMTP authentication username.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// SMTP authentication password.
    #[must_use]
    pub fn password(&self) -> &str {
        &self.password
    }

    /// The sender address used in outgoing emails.
    #[must_use]
    pub fn from_address(&self) -> &str {
        &self.from_address
    }

    /// Whether to use STARTTLS when connecting to the SMTP server.
    #[must_use]
    pub const fn use_tls(&self) -> bool {
        self.use_tls
    }
}
