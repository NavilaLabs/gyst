use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;
use zeitrak_infrastructure::email::EmailSender;
use zeitrak_infrastructure_impl::Pool;

/// Adds an email address to the waitlist using the globally configured admin pool.
///
/// Silently ignores duplicate submissions (same email already present).
/// On a new sign-up, sends a notification to the owner and a confirmation to
/// the subscriber.  Email delivery failures are logged as warnings and do not
/// prevent the record from being persisted.
pub async fn join_waitlist(
    email: String,
    email_sender: &dyn EmailSender,
    owner_email: &str,
) -> Result<()> {
    let pool = Pool::connect_admin().await?;
    join_waitlist_on(pool, email, email_sender, owner_email).await
}

/// Same as [`join_waitlist`] but operates on a caller-supplied pool (useful in tests).
pub async fn join_waitlist_on(
    pool: zeitrak_infrastructure_impl::ConnectedAdminPool,
    email: String,
    email_sender: &dyn EmailSender,
    owner_email: &str,
) -> Result<()> {
    let id = Uuid::now_v7().to_string();
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
        "INSERT OR IGNORE INTO waitlist_signups (id, email, created_at) VALUES (?, ?, ?)",
    )
    .bind(&id)
    .bind(&email)
    .bind(&now)
    .execute(pool.as_ref())
    .await?;

    // rows_affected == 0 means the email was already in the list — skip notifications.
    if result.rows_affected() == 0 {
        return Ok(());
    }

    if !owner_email.is_empty()
        && let Err(e) = email_sender
            .send_waitlist_notification(owner_email, &email)
            .await
    {
        tracing::warn!(error = %e, "failed to send waitlist notification for {email}");
    }

    if let Err(e) = email_sender.send_waitlist_confirmation(&email).await {
        tracing::warn!(error = %e, "failed to send waitlist confirmation to {email}");
    }

    Ok(())
}
