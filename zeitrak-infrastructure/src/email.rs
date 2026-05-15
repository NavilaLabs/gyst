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
}
