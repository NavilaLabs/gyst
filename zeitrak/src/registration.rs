use anyhow::Result;
use zeitrak_core::admin::user::{
    CreateUserInput, UserCommand, UserCommandTrait, UserId, UserQuery, UserQueryTrait,
};
use zeitrak_infrastructure_impl::{
    Pool, admin::authentication::hash_password, admin::user::repositories::UserRepository,
};

use crate::error::validate;

/// Creates a new user account.
///
/// Returns an error if the email is already taken or input validation fails.
pub async fn register_user(name: String, email: String, password: String) -> Result<UserId> {
    validate(&CreateUserInput {
        name: name.clone(),
        email: email.clone(),
    })?;

    let pool = Pool::connect_admin().await?;
    let repo = UserRepository::from_pool(pool.clone()).await?;

    if UserQuery::new(repo)
        .find_id_by_email(&email)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .is_some()
    {
        anyhow::bail!("email already registered");
    }

    let hashed = hash_password(&password)?;
    let user_id = UserId::new();
    let _ = UserCommand::new(UserRepository::from_pool(pool).await?)
        .create(user_id.clone(), name, email, hashed)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(user_id)
}
