use async_trait::async_trait;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use zeitrak_infrastructure::email::EmailSender;

/// Configuration for the SMTP email transport.
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    /// When `false`, connect over plain SMTP without TLS (e.g. MailHog).
    pub use_tls: bool,
}

/// SMTP-backed implementation of [`EmailSender`].
pub struct SmtpEmailSender {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from_address: String,
}

impl SmtpEmailSender {
    /// Create a new sender from the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the SMTP connection cannot be established.
    pub fn new(config: SmtpConfig) -> anyhow::Result<Self> {
        let transport = if config.use_tls {
            let credentials = Credentials::new(config.username, config.password);
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)?
                .port(config.port)
                .credentials(credentials)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
                .port(config.port)
                .build()
        };
        Ok(Self {
            transport,
            from_address: config.from_address,
        })
    }
}

#[async_trait]
impl EmailSender for SmtpEmailSender {
    async fn send_invitation(
        &self,
        to: &str,
        invitation_link: &str,
        workspace_name: &str,
        invited_by_name: &str,
    ) -> anyhow::Result<()> {
        let body = format!(
            "{invited_by_name} has invited you to join the workspace \"{workspace_name}\" on Zeitrak.\n\n\
             Accept the invitation here:\n{invitation_link}\n\n\
             This link expires in 7 days."
        );

        let email = Message::builder()
            .from(self.from_address.parse()?)
            .to(to.parse()?)
            .subject(format!(
                "You're invited to join {workspace_name} on Zeitrak"
            ))
            .header(ContentType::TEXT_PLAIN)
            .body(body)?;

        self.transport.send(email).await?;
        Ok(())
    }

    async fn send_verification_email(
        &self,
        to: &str,
        verification_link: &str,
    ) -> anyhow::Result<()> {
        let body = format!(
            "Welcome to Zeitrak! Please verify your email address by clicking the link below:\n\n\
             {verification_link}\n\n\
             This link expires in 24 hours. If you did not register, you can safely ignore this email."
        );

        let email = Message::builder()
            .from(self.from_address.parse()?)
            .to(to.parse()?)
            .subject("Verify your Zeitrak account")
            .header(ContentType::TEXT_PLAIN)
            .body(body)?;

        self.transport.send(email).await?;
        Ok(())
    }
}
