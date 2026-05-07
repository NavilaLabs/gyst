use anyhow::Result;
use eventually::aggregate::{Root, repository::Saver};
use zeitrak_core::admin::{
    user::{UserEvent, UserId, UserRepository as UserRepositoryTrait},
    workspace::{Workspace, WorkspaceEvent, WorkspaceId},
    workspace_role::{WorkspaceRole, WorkspaceRoleEvent, WorkspaceRoleId},
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
    let user_repo = UserRepository::from_pool(pool).await?;
    Ok(user_repo.has_at_least_one_user().await?)
}

pub async fn setup_application(
    username: String,
    email: String,
    password: String,
    workspace_name: String,
) -> Result<()> {
    let pool = Pool::connect_admin().await?;

    let user_repo = UserRepository::from_pool(pool.clone()).await?;

    if user_repo.has_at_least_one_user().await? {
        anyhow::bail!("application is already set up");
    }

    // 1. Create the admin user.
    let password = hash_password(&password)?;
    let user_id = UserId::new();
    let mut user_root = Root::<zeitrak_core::admin::user::User>::record_new(
        UserEvent::Created {
            id: user_id.clone(),
            name: username,
            email,
            password,
        }
        .into(),
    )?;
    user_repo.save(&mut user_root).await?;

    // 2. Create the workspace (save first so the projection row exists before the role).
    let workspace_id = WorkspaceId::new();
    let workspace_repo = WorkspaceRepository::from_pool(pool.clone()).await?;
    let mut workspace_root = Root::<Workspace>::record_new(
        WorkspaceEvent::Created {
            id: workspace_id.clone(),
            name: Some(workspace_name),
        }
        .into(),
    )?;
    workspace_repo.save(&mut workspace_root).await?;

    // 3. Create the "admin" role for this workspace.
    let role_repo = WorkspaceRoleRepository::from_pool(pool.clone()).await?;
    let role_id = WorkspaceRoleId::new();
    let mut role_root = Root::<WorkspaceRole>::record_new(
        WorkspaceRoleEvent::Created {
            id: role_id.clone(),
            workspace_id: workspace_id.clone(),
            name: Some("admin".to_string()),
        }
        .into(),
    )?;
    role_repo.save(&mut role_root).await?;

    // 4. Assign the user to the workspace with the admin role.
    workspace_root.record_that(
        WorkspaceEvent::UserRoleAssigned {
            user_id,
            workspace_role_id: role_id,
        }
        .into(),
    )?;
    workspace_repo.save(&mut workspace_root).await?;

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
