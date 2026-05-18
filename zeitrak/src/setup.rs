use anyhow::Result;
use zeitrak_core::admin::user::{UserCommand, UserCommandTrait, UserId, UserQuery, UserQueryTrait};
use zeitrak_infrastructure::database::Migrate;
use zeitrak_infrastructure_impl::{
    Pool, ScopeDefault, StateDisconnected, admin::{authentication::hash_password, user::repositories::UserRepository}, database::{Initializer, SqliteInitializationStrategy}
};

use crate::workspace::create_workspace_for_user;

/// Ensures the admin `SQLite` file exists and all migrations are up to date.
/// Call once at server startup before accepting requests.
pub async fn init_admin_db() -> Result<()> {
    // Create the file if it doesn't exist.
    let default_pool = Pool::<ScopeDefault, StateDisconnected>::connect_default().await?;
    let ini = Initializer::new(SqliteInitializationStrategy);
    ini.initialize_admin(&default_pool).await?;

    // Run pending migrations.
    let admin_pool = Pool::connect_admin().await?;
    admin_pool.migrate_database().await?;

    Ok(())
}

/// Returns `true` if at least one user exists, meaning setup has already been run.
pub async fn is_setup_complete() -> Result<bool> {
    let pool = Pool::connect_admin().await?;
    let repo = UserRepository::from_pool(pool).await?;
    UserQuery::new(repo)
        .has_at_least_one_user()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

pub async fn setup_application(
    username: String,
    email: String,
    password: String,
    workspace_name: String,
) -> Result<()> {
    if is_setup_complete().await? {
        anyhow::bail!("application is already set up");
    }
    let pool = Pool::connect_admin().await?;

    let user_repo = UserRepository::from_pool(pool.clone()).await?;
    if UserQuery::new(user_repo)
        .has_at_least_one_user()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?
    {
        anyhow::bail!("application is already set up");
    }

    // 1. Create the admin user.
    let password = hash_password(&password)?;
    let user_id = UserId::new();
    let _ = UserCommand::new(UserRepository::from_pool(pool.clone()).await?)
        .create(user_id.clone(), username, email, password)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // 2–5. Create the workspace, admin role, assign user, init tenant DB.
    create_workspace_for_user(user_id, workspace_name).await?;

    Ok(())
}
