use async_trait::async_trait;

/// Sends transactional emails on behalf of the application.
///
/// The concrete implementation is provided by the infrastructure-impl layer
/// and injected at startup.  The domain layer only sees this trait.
#[async_trait]
pub trait EmailSender: Send + Sync {
    /// Sends a workspace invitation email.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying transport fails.
    async fn send_invitation(
        &self,
        to: &str,
        invitation_link: &str,
        workspace_name: &str,
        invited_by_name: &str,
        ttl_days: u32,
    ) -> anyhow::Result<()>;

    /// Sends an email verification link to a newly registered user.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying transport fails.
    async fn send_verification_email(
        &self,
        to: &str,
        verification_link: &str,
    ) -> anyhow::Result<()>;

    /// Notifies the application owner that a new address joined the waitlist.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying transport fails.
    async fn send_waitlist_notification(
        &self,
        owner_email: &str,
        subscriber_email: &str,
    ) -> anyhow::Result<()>;

    /// Sends a confirmation email to a waitlist subscriber.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying transport fails.
    async fn send_waitlist_confirmation(&self, to: &str) -> anyhow::Result<()>;
}

/// The authentication method stored in the `smtp_config` admin-DB row.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SmtpAuthMethod {
    /// Username + password (STARTTLS or plain).
    Password,
    /// Microsoft 365 XOAUTH2 via refresh-token flow.
    XOAuth2,
}

/// SMTP configuration as persisted in the admin database.
///
/// Sensitive fields (`password`, `client_secret`, `refresh_token`) are
/// decrypted by the repository layer before populating this struct.
#[derive(Debug, Clone)]
pub struct PersistedSmtpConfig {
    /// Authentication mechanism to use.
    pub auth_method: SmtpAuthMethod,
    /// SMTP server hostname.
    pub host: String,
    /// SMTP server port.
    pub port: u16,
    /// SMTP login username (also used as the `OAuth2` SMTP address).
    pub username: String,
    /// Sender address used in outgoing emails.
    pub from_address: String,
    /// Whether to use STARTTLS.
    pub use_tls: bool,
    /// Plaintext password (password auth only).
    pub password: Option<String>,
    /// `OAuth2` application (client) ID.
    pub client_id: Option<String>,
    /// `OAuth2` client secret (decrypted).
    pub client_secret: Option<String>,
    /// Azure AD tenant ID.
    pub tenant_id: Option<String>,
    /// Long-lived refresh token (decrypted).
    pub refresh_token: Option<String>,
    /// The Microsoft 365 mailbox address used for SMTP AUTH.
    pub oauth2_smtp_email: Option<String>,
    /// True once the `OAuth2` authorization code flow has completed.
    pub oauth2_authorized: bool,
}

/// Repository for the single-row SMTP configuration in the admin database.
#[async_trait]
pub trait SmtpConfigRepository: Send + Sync {
    /// Returns the persisted SMTP config, or `None` if it has never been saved.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query or decryption fails.
    async fn get(&self) -> anyhow::Result<Option<PersistedSmtpConfig>>;

    /// Upserts the SMTP config row.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write or encryption fails.
    async fn save(&self, config: &PersistedSmtpConfig) -> anyhow::Result<()>;

    /// Stores a CSRF state token before initiating the `OAuth2` authorization flow.
    ///
    /// # Errors
    ///
    /// Returns an error if no `smtp_config` row exists yet or the update fails.
    async fn set_oauth2_state(&self, state: &str) -> anyhow::Result<()>;

    /// Validates `actual_state` against the stored CSRF token, then persists the
    /// `refresh_token`, marks `oauth2_authorized = true`, and clears `oauth2_state`.
    ///
    /// # Errors
    ///
    /// Returns an error if the state does not match or the database write fails.
    async fn complete_oauth2(&self, actual_state: &str, refresh_token: &str) -> anyhow::Result<()>;
}
