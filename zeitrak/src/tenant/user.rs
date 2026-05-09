use anyhow::Result;
use zeitrak_core::admin::user::{UserCommand, UserCommandTrait, UserId};
use zeitrak_infrastructure_impl::{Pool, admin::user::repositories::UserRepository};

/// Create a new user and persist it to the admin store.
pub async fn create(id: UserId, name: String, email: String, password: String) -> Result<()> {
    let pool = Pool::connect_admin().await?;
    let repository = UserRepository::from_pool(pool).await?;
    let _ = UserCommand::new(repository)
        .create(id, name, email, password)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}
