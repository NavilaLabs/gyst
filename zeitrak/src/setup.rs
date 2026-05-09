use anyhow::Result;
use zeitrak_core::admin::{
    user::{UserCommand, UserCommandTrait, UserQuery, UserQueryTrait, UserId},
    workspace::{WorkspaceCommand, WorkspaceCommandTrait, WorkspaceId},
    workspace_role::{WorkspaceRoleCommand, WorkspaceRoleCommandTrait, WorkspaceRoleId},
};
use zeitrak_infrastructure::database::Migrate;
use zeitrak_infrastructure_impl::{
    Pool, ScopeDefault, ScopeTenant, StateDisconnected,
    admin::{
        authentication::hash_password, user::repositories::UserRepository,
        workspace::repositories::WorkspaceRepository,
        workspace_role::repositories::WorkspaceRoleRepository,
    },
    database::{Initializer, SqliteInitializationStrategy},
};

/// Ensures the admin `SQLite` file exists and all migrations are up to date.
/// Call once at server startup before accepting requests.
pub async fn init_admin_db() -> Result<()> {
    // Create the file if it doesn't exist.
    let default_pool = Pool::<ScopeDefault, StateDisconnected>::connect_default().await?;
    Initializer::new(SqliteInitializationStrategy)
        .initialize_admin(&default_pool)
        .await?;

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

    // 2. Create the workspace.
    let workspace_id = WorkspaceId::new();
    let _ = WorkspaceCommand::new(WorkspaceRepository::from_pool(pool.clone()).await?)
        .create(workspace_id.clone(), Some(workspace_name))
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // 3. Create the "admin" role for this workspace.
    let role_id = WorkspaceRoleId::new();
    let _ = WorkspaceRoleCommand::new(WorkspaceRoleRepository::from_pool(pool.clone()).await?)
        .create(role_id.clone(), workspace_id.clone(), Some("admin".to_string()))
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // 4. Assign the user to the workspace with the admin role.
    WorkspaceCommand::new(WorkspaceRepository::from_pool(pool.clone()).await?)
        .assign_user_role(workspace_id.clone(), user_id, role_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // 5. Create and migrate the tenant database for this workspace.
    let tenant_token = workspace_id.to_string();
    let default_pool = Pool::<ScopeDefault, StateDisconnected>::connect_default().await?;
    Initializer::new(SqliteInitializationStrategy)
        .initialize_tenant(&default_pool, Some(&tenant_token))
        .await?;
    let tenant_pool = Pool::<ScopeTenant, StateDisconnected>::connect_tenant(&tenant_token).await?;
    tenant_pool.migrate_database().await?;

    Ok(())
}
