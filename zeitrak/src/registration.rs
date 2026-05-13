use anyhow::Result;
use uuid::Uuid;
use zeitrak_core::admin::user::{
    CreateUserInput, UserCommand, UserCommandTrait, UserId, UserQuery, UserQueryTrait,
};
use zeitrak_infrastructure::config::CONFIG;
use zeitrak_infrastructure::email::EmailSender;
use zeitrak_infrastructure_impl::{
    ConnectedAdminPool, Pool, admin::authentication::hash_password,
    admin::user::repositories::UserRepository,
};

use crate::error::validate;

/// Returns `true` when registration is restricted to invited users only.
#[must_use]
pub fn is_invite_only() -> bool {
    CONFIG.application().security().invite_only()
}

/// Creates a new user account using the globally configured admin pool.
///
/// Convenience wrapper around [`register_user_on`] for production use.
pub async fn register_user(
    name: String,
    email: String,
    password: String,
    email_sender: &dyn EmailSender,
    base_url: &str,
) -> Result<UserId> {
    let pool = Pool::connect_admin().await?;
    register_user_on(pool, name, email, password, email_sender, base_url).await
}

/// Creates a new user account on the given `pool` and sends a verification email.
///
/// Prefer this form in tests so an isolated [`TestFixture`](zeitrak_tests::TestFixture)
/// pool can be supplied instead of the globally-configured one.
///
/// Returns an error if the email is already taken, input validation fails, or
/// the event store cannot be written. Email delivery failures are logged as
/// warnings but do not prevent the account from being created.
pub async fn register_user_on(
    pool: ConnectedAdminPool,
    name: String,
    email: String,
    password: String,
    email_sender: &dyn EmailSender,
    base_url: &str,
) -> Result<UserId> {
    validate(&CreateUserInput {
        name: name.clone(),
        email: email.clone(),
        password: password.clone(),
    })?;

    let repo = UserRepository::from_pool(pool.clone()).await?;
    let query = UserQuery::new(repo);

    if let Some(existing_id) = query
        .find_id_by_email(&email)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?
    {
        // Allow re-registration only when the account has not been verified yet.
        // This lets users who never clicked (or whose link expired) get a fresh token
        // by filling in the registration form again.
        let view = query
            .find_view_by_id(&existing_id.to_string())
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?
            .ok_or_else(|| anyhow::anyhow!("user projection missing for {existing_id}"))?;

        if view.is_verified {
            anyhow::bail!("email already registered");
        }

        let token = Uuid::now_v7().to_string();
        UserCommand::new(UserRepository::from_pool(pool).await?)
            .request_verification(existing_id.clone(), token.clone())
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let link = format!("{base_url}/verify-email/{token}");
        if let Err(e) = email_sender.send_verification_email(&email, &link).await {
            tracing::warn!(error = %e, "failed to resend verification email to {email}");
        }

        return Ok(existing_id);
    }

    let hashed = hash_password(&password)?;
    let user_id = UserId::new();
    let _ = UserCommand::new(UserRepository::from_pool(pool.clone()).await?)
        .create(user_id.clone(), name, email.clone(), hashed)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let token = Uuid::now_v7().to_string();
    UserCommand::new(UserRepository::from_pool(pool).await?)
        .request_verification(user_id.clone(), token.clone())
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let link = format!("{base_url}/verify-email/{token}");
    if let Err(e) = email_sender.send_verification_email(&email, &link).await {
        tracing::warn!(error = %e, "failed to send verification email to {email}");
    }

    Ok(user_id)
}

/// Verifies a user's email address using the globally configured admin pool.
pub async fn verify_email_by_token(token: &str) -> Result<UserId> {
    let pool = Pool::connect_admin().await?;
    verify_email_by_token_on(pool, token).await
}

/// Verifies a user's email address on the given `pool`.
///
/// Returns the `UserId` on success. Returns an error if the token is invalid
/// or has already been consumed.
pub async fn verify_email_by_token_on(pool: ConnectedAdminPool, token: &str) -> Result<UserId> {
    let repo = UserRepository::from_pool(pool.clone()).await?;

    let user_id = UserQuery::new(repo)
        .find_id_by_verification_token(token)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .ok_or_else(|| anyhow::anyhow!("invalid or expired verification token"))?;

    UserCommand::new(UserRepository::from_pool(pool).await?)
        .verify_email(user_id.clone())
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(user_id)
}
