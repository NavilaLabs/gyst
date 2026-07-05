use anyhow::Result;
use zeitrak_infrastructure::{
    config::CONFIG,
    email::{EmailSender, PersistedSmtpConfig, SmtpAuthMethod, SmtpConfigRepository as _},
};
use zeitrak_infrastructure_impl::{
    Pool,
    smtp::{OAuth2SmtpEmailSender, SmtpConfig, SmtpConfigRepositoryImpl, SmtpEmailSender},
};

/// Returns an [`EmailSender`] for transactional email delivery.
///
/// Resolution order:
/// 1. Admin-database row (`smtp_config` table) — includes encrypted password
///    or `OAuth2` refresh token.
/// 2. Static `CONFIG` (env vars / YAML files).
///
/// # Errors
///
/// Returns an error if the admin database is unreachable or if neither the
/// database nor the static config has a usable SMTP configuration.
pub async fn email_sender_from_config() -> Result<Box<dyn EmailSender>> {
    let pool = Pool::connect_admin().await?;
    let secret = CONFIG.application().security().authentication_secret();
    let repo = SmtpConfigRepositoryImpl::new(pool, secret);

    if let Some(persisted) = repo.get().await? {
        return build_from_persisted(persisted);
    }

    build_from_static_config()
}

fn build_from_persisted(p: PersistedSmtpConfig) -> Result<Box<dyn EmailSender>> {
    match p.auth_method {
        SmtpAuthMethod::Password => {
            let sender = SmtpEmailSender::new(SmtpConfig {
                host: p.host,
                port: p.port,
                username: p.username,
                password: p.password.unwrap_or_default(),
                from_address: p.from_address,
                use_tls: p.use_tls,
            })?;
            Ok(Box::new(sender))
        }
        SmtpAuthMethod::XOAuth2 if p.oauth2_authorized => {
            let sender = OAuth2SmtpEmailSender::new(p)?;
            Ok(Box::new(sender))
        }
        SmtpAuthMethod::XOAuth2 => {
            anyhow::bail!(
                "OAuth2 SMTP is configured but not yet authorized — complete the Microsoft authorization flow first"
            )
        }
        _ => build_from_static_config(),
    }
}

fn build_from_static_config() -> Result<Box<dyn EmailSender>> {
    let smtp = CONFIG.application().smtp();
    let sender = SmtpEmailSender::new(SmtpConfig {
        host: smtp.host().to_string(),
        port: smtp.port(),
        username: smtp.username().to_string(),
        password: smtp.password().to_string(),
        from_address: smtp.from_address().to_string(),
        use_tls: smtp.use_tls(),
    })?;
    Ok(Box::new(sender))
}

/// Returns the base URL for building invitation links.
#[must_use]
pub fn base_url() -> &'static str {
    CONFIG.application().base_url()
}

/// Returns the owner email address for waitlist notifications.
#[must_use]
pub fn owner_email() -> &'static str {
    CONFIG.application().owner_email()
}
