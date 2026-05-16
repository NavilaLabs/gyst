use anyhow::Result;
use zeitrak_infrastructure::config::CONFIG;
pub use zeitrak_infrastructure::email::EmailSender;
use zeitrak_infrastructure_impl::smtp::{SmtpConfig, SmtpEmailSender};

/// Creates an [`SmtpEmailSender`] from the application configuration.
///
/// Returns an error if SMTP is not configured or the transport cannot be
/// initialized.
///
/// # Errors
///
/// Returns an error if the SMTP configuration is missing or invalid.
pub fn email_sender_from_config() -> Result<impl EmailSender> {
    let smtp_cfg = CONFIG.application().smtp();

    SmtpEmailSender::new(SmtpConfig {
        host: smtp_cfg.host().to_string(),
        port: smtp_cfg.port(),
        username: smtp_cfg.username().to_string(),
        password: smtp_cfg.password().to_string(),
        from_address: smtp_cfg.from_address().to_string(),
        use_tls: smtp_cfg.use_tls(),
    })
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
