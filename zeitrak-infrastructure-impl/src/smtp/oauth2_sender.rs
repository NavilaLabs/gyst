use async_trait::async_trait;
use chrono::{Duration, Utc};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::header::ContentType,
    transport::smtp::authentication::{Credentials, Mechanism},
};
use serde::Deserialize;
use zeitrak_infrastructure::email::{EmailSender, PersistedSmtpConfig};

use crate::smtp::oauth2_cache::{CachedToken, TOKEN_CACHE};

/// Microsoft OAuth2 token endpoint response (subset of fields we use).
#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

/// SMTP email sender that authenticates via Microsoft XOAUTH2.
///
/// The access token is refreshed automatically using the stored refresh token.
/// A process-wide in-memory cache avoids redundant token exchanges.
pub struct OAuth2SmtpEmailSender {
    host: String,
    port: u16,
    use_tls: bool,
    from_address: String,
    smtp_email: String,
    client_id: String,
    client_secret: String,
    tenant_id: String,
    refresh_token: String,
}

impl OAuth2SmtpEmailSender {
    /// Creates a new sender from a fully authorized [`PersistedSmtpConfig`].
    ///
    /// # Errors
    ///
    /// Returns an error if any required OAuth2 field is missing.
    pub fn new(config: PersistedSmtpConfig) -> anyhow::Result<Self> {
        Ok(Self {
            host: config.host,
            port: config.port,
            use_tls: config.use_tls,
            from_address: config.from_address,
            smtp_email: config
                .oauth2_smtp_email
                .ok_or_else(|| anyhow::anyhow!("oauth2_smtp_email is required"))?,
            client_id: config
                .client_id
                .ok_or_else(|| anyhow::anyhow!("oauth2 client_id is required"))?,
            client_secret: config
                .client_secret
                .ok_or_else(|| anyhow::anyhow!("oauth2 client_secret is required"))?,
            tenant_id: config
                .tenant_id
                .ok_or_else(|| anyhow::anyhow!("oauth2 tenant_id is required"))?,
            refresh_token: config
                .refresh_token
                .ok_or_else(|| anyhow::anyhow!("oauth2 refresh_token is required"))?,
        })
    }

    /// Returns a valid access token, refreshing it via the token endpoint if needed.
    async fn access_token(&self) -> anyhow::Result<String> {
        {
            let cache = TOKEN_CACHE.lock().await;
            if let Some(ref cached) = *cache {
                // Reuse if the token is valid for at least another 60 seconds.
                if cached.expires_at > Utc::now() + Duration::seconds(60) {
                    return Ok(cached.token.clone());
                }
            }
        }

        let token = self.refresh_access_token().await?;
        Ok(token)
    }

    async fn refresh_access_token(&self) -> anyhow::Result<String> {
        let url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            self.tenant_id
        );

        let params = [
            ("grant_type", "refresh_token"),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("refresh_token", &self.refresh_token),
            ("scope", "https://outlook.office.com/SMTP.Send offline_access"),
        ];

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .form(&params)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("token request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("token endpoint returned {status}: {body}");
        }

        let token_resp: TokenResponse = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("failed to parse token response: {e}"))?;

        let expires_at = Utc::now()
            + Duration::seconds(
                i64::try_from(token_resp.expires_in).unwrap_or(3600),
            );

        let mut cache = TOKEN_CACHE.lock().await;
        *cache = Some(CachedToken {
            token: token_resp.access_token.clone(),
            expires_at,
        });

        Ok(token_resp.access_token)
    }

    async fn build_transport(&self) -> anyhow::Result<AsyncSmtpTransport<Tokio1Executor>> {
        let access_token = self.access_token().await?;
        let credentials = Credentials::new(self.smtp_email.clone(), access_token);

        let transport = if self.use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.host)?
                .port(self.port)
                .credentials(credentials)
                .authentication(vec![Mechanism::Xoauth2])
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.host)
                .port(self.port)
                .credentials(credentials)
                .authentication(vec![Mechanism::Xoauth2])
                .build()
        };

        Ok(transport)
    }

    async fn send_message(&self, message: Message) -> anyhow::Result<()> {
        let transport = self.build_transport().await?;
        transport.send(message).await?;
        Ok(())
    }
}

#[async_trait]
impl EmailSender for OAuth2SmtpEmailSender {
    async fn send_invitation(
        &self,
        to: &str,
        invitation_link: &str,
        workspace_name: &str,
        invited_by_name: &str,
        ttl_days: u32,
    ) -> anyhow::Result<()> {
        let body = format!(
            "{invited_by_name} has invited you to join the workspace \"{workspace_name}\" on Zeitrak.\n\n\
             Accept the invitation here:\n{invitation_link}\n\n\
             This link expires in {ttl_days} days."
        );
        let email = Message::builder()
            .from(self.from_address.parse()?)
            .to(to.parse()?)
            .subject(format!("You're invited to join {workspace_name} on Zeitrak"))
            .header(ContentType::TEXT_PLAIN)
            .body(body)?;
        self.send_message(email).await
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
        self.send_message(email).await
    }

    async fn send_waitlist_notification(
        &self,
        owner_email: &str,
        subscriber_email: &str,
    ) -> anyhow::Result<()> {
        let body = format!(
            "Someone joined the Zeitrak early access waitlist: {subscriber_email}"
        );
        let email = Message::builder()
            .from(self.from_address.parse()?)
            .to(owner_email.parse()?)
            .subject("New Zeitrak waitlist sign-up")
            .header(ContentType::TEXT_PLAIN)
            .body(body)?;
        self.send_message(email).await
    }

    async fn send_waitlist_confirmation(&self, to: &str) -> anyhow::Result<()> {
        let body = "Thanks for your interest in Zeitrak!\n\n\
             We'll reach out as soon as early access opens.\n\n\
             The Zeitrak team";
        let email = Message::builder()
            .from(self.from_address.parse()?)
            .to(to.parse()?)
            .subject("You're on the Zeitrak early access list")
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())?;
        self.send_message(email).await
    }
}
