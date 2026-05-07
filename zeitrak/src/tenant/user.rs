use anyhow::Result;
use eventually::aggregate::{Root, repository::Saver};
use zeitrak_core::admin::user::{User, UserCommand, UserCommandTrait, UserId};
use zeitrak_infrastructure_impl::{Pool, admin::user::repositories::UserRepository};

/// Create a new user and persist it to the admin store.
pub async fn create(id: UserId, name: String, email: String, password: String) -> Result<()> {
    let pool = Pool::connect_admin().await?;
    let repository = UserRepository::from_pool(pool).await?;
    let mut user = UserCommand::new(repository.clone()).create(id, name, email, password).await?;
    repository.save(&mut user).await?;
    Ok(())
}
