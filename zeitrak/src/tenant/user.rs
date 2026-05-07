use anyhow::Result;
use eventually::aggregate::repository::Saver;
use zeitrak_core::admin::user::{UserCommand, UserId};
use zeitrak_infrastructure_impl::{Pool, admin::user::repositories::UserRepository};

/// Create a new user and persist it to the admin store.
pub async fn create(id: UserId, name: String, email: String, password: String) -> Result<()> {
    let pool = Pool::connect_admin().await?;
    let repo = UserRepository::from_pool(pool).await?;
    let mut cmd = UserCommand::create(id, name, email, password)?;
    repo.save(&mut cmd).await?;
    Ok(())
}
