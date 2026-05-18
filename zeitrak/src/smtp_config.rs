use anyhow::Result;
use zeitrak_infrastructure::{
    config::CONFIG,
    email::{PersistedSmtpConfig, SmtpAuthMethod, SmtpConfigRepository as _},
};
use zeitrak_infrastructure_impl::{Pool, smtp::SmtpConfigRepositoryImpl};

/// SMTP configuration data returned to the presentation layer.
///
/// Sensitive fields (passwords, secrets) are masked — only their presence is
/// indicated via boolean flags.
#[derive(Debug, Clone)]
pub struct SmtpConfigDto {
    /// `"password"` or `"xoauth2"`.
    pub auth_method: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub from_address: String,
    pub use_tls: bool,
    /// `true` if a password is stored (the value is never returned).
    pub password_is_set: bool,
    pub client_id: Option<String>,
    pub tenant_id: Option<String>,
    /// `true` if a client secret is stored.
    pub client_secret_is_set: bool,
    pub oauth2_smtp_email: Option<String>,
    /// `true` once the OAuth2 authorization code flow has completed.
    pub oauth2_authorized: bool,
}

fn repo_from_pool(pool: zeitrak_infrastructure_impl::ConnectedAdminPool) -> SmtpConfigRepositoryImpl {
    let secret = CONFIG.application().security().authentication_secret();
    SmtpConfigRepositoryImpl::new(pool, secret)
}

/// Returns the current SMTP configuration, DB-first with a fallback to the
/// static `CONFIG`.
///
/// # Errors
///
/// Returns an error if the admin database cannot be reached.
pub async fn get_smtp_config_dto() -> Result<SmtpConfigDto> {
    let pool = Pool::connect_admin().await?;
    let repo = repo_from_pool(pool);

    if let Some(p) = repo.get().await? {
        return Ok(persisted_to_dto(p));
    }

    // Fallback to static CONFIG
    let smtp = CONFIG.application().smtp();
    Ok(SmtpConfigDto {
        auth_method: "password".to_string(),
        host: smtp.host().to_string(),
        port: smtp.port(),
        username: smtp.username().to_string(),
        from_address: smtp.from_address().to_string(),
        use_tls: smtp.use_tls(),
        password_is_set: !smtp.password().is_empty(),
        client_id: None,
        tenant_id: None,
        client_secret_is_set: false,
        oauth2_smtp_email: None,
        oauth2_authorized: false,
    })
}

/// Saves SMTP configuration to the admin database.
///
/// If `password` / `client_secret` are `None`, the existing encrypted value is
/// preserved.  Set them to `Some("")` to explicitly clear a stored secret.
///
/// # Errors
///
/// Returns an error if the admin database cannot be reached or encryption fails.
pub async fn save_smtp_config(
    auth_method: String,
    host: String,
    port: u16,
    username: String,
    from_address: String,
    use_tls: bool,
    password: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    tenant_id: Option<String>,
    oauth2_smtp_email: Option<String>,
) -> Result<()> {
    let pool = Pool::connect_admin().await?;
    let repo = repo_from_pool(pool);

    // Load existing row to preserve encrypted secrets when callers send None.
    let existing = repo.get().await?;

    let resolved_password = match password {
        Some(p) => Some(p),
        None => existing.as_ref().and_then(|e| e.password.clone()),
    };
    let resolved_secret = match client_secret {
        Some(s) => Some(s),
        None => existing.as_ref().and_then(|e| e.client_secret.clone()),
    };

    let parsed_auth_method = if auth_method == "xoauth2" {
        SmtpAuthMethod::XOAuth2
    } else {
        SmtpAuthMethod::Password
    };

    // Preserve oauth2_authorized and refresh_token from the existing row when
    // the caller does not clear them.
    let oauth2_authorized = existing
        .as_ref()
        .map_or(false, |e| e.oauth2_authorized);
    let refresh_token = existing.as_ref().and_then(|e| e.refresh_token.clone());

    let config = PersistedSmtpConfig {
        auth_method: parsed_auth_method,
        host,
        port,
        username,
        from_address,
        use_tls,
        password: resolved_password,
        client_id,
        client_secret: resolved_secret,
        tenant_id,
        refresh_token,
        oauth2_smtp_email,
        oauth2_authorized,
    };

    repo.save(&config).await?;
    Ok(())
}

/// Sends a test email to `to_address` using the current SMTP configuration.
///
/// # Errors
///
/// Returns an error if the SMTP configuration is missing or the transport fails.
pub async fn test_smtp_connection(to_address: String) -> Result<()> {
    let sender = crate::email::email_sender_from_config().await?;
    sender
        .send_verification_email(
            &to_address,
            "https://example.com/test — this is a Zeitrak SMTP test email",
        )
        .await
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn persisted_to_dto(p: PersistedSmtpConfig) -> SmtpConfigDto {
    SmtpConfigDto {
        auth_method: match p.auth_method {
            SmtpAuthMethod::Password => "password".to_string(),
            SmtpAuthMethod::XOAuth2 => "xoauth2".to_string(),
            _ => "password".to_string(),
        },
        host: p.host,
        port: p.port,
        username: p.username,
        from_address: p.from_address,
        use_tls: p.use_tls,
        password_is_set: p.password.is_some(),
        client_id: p.client_id,
        tenant_id: p.tenant_id,
        client_secret_is_set: p.client_secret.is_some(),
        oauth2_smtp_email: p.oauth2_smtp_email,
        oauth2_authorized: p.oauth2_authorized,
    }
}
