use anyhow::Result;
use serde::{Deserialize, Serialize};
use zeitrak_core::admin::{
    user::UserId,
    workspace::{
        WorkspaceCommand, WorkspaceCommandTrait, WorkspaceId, WorkspaceQuery, WorkspaceQueryTrait,
        WorkspaceRow,
    },
    workspace_role::{
        WorkspaceRoleCommand, WorkspaceRoleCommandTrait, WorkspaceRoleId, WorkspaceRoleRow,
    },
};
use zeitrak_core::shared::repositories::ReadRepository;
use zeitrak_infrastructure::database::Migrate;
use zeitrak_infrastructure_impl::{
    Pool, ScopeDefault, ScopeTenant, StateDisconnected,
    admin::{
        workspace::repositories::WorkspaceRepository,
        workspace_role::repositories::WorkspaceRoleRepository,
    },
    database::{Initializer, SqliteInitializationStrategy},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub id: String,
    pub name: Option<String>,
}

/// Creates a new workspace for an existing user and initialises its tenant database.
///
/// This is the post-registration counterpart to `setup_application`: it skips
/// user creation and sets up the workspace, admin role, and tenant DB only.
pub async fn create_workspace_for_user(
    user_id: UserId,
    workspace_name: String,
) -> Result<WorkspaceId> {
    let pool = Pool::connect_admin().await?;

    let workspace_id = WorkspaceId::new();
    let _ = WorkspaceCommand::new(WorkspaceRepository::from_pool(pool.clone()).await?)
        .create(workspace_id.clone(), Some(workspace_name))
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let role_id = WorkspaceRoleId::new();
    let _ = WorkspaceRoleCommand::new(WorkspaceRoleRepository::from_pool(pool.clone()).await?)
        .create(
            role_id.clone(),
            workspace_id.clone(),
            Some("admin".to_string()),
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    WorkspaceCommand::new(WorkspaceRepository::from_pool(pool).await?)
        .assign_user_role(workspace_id.clone(), user_id, role_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let tenant_token = workspace_id.to_string();
    let default_pool = Pool::<ScopeDefault, StateDisconnected>::connect_default().await?;
    Initializer::new(SqliteInitializationStrategy)
        .initialize_tenant(&default_pool, Some(&tenant_token))
        .await?;
    let tenant_pool = Pool::<ScopeTenant, StateDisconnected>::connect_tenant(&tenant_token).await?;
    tenant_pool.migrate_database().await?;

    Ok(workspace_id)
}

/// Returns all workspaces the given user is a member of.
pub async fn list_user_workspaces(user_id: &str) -> Result<Vec<WorkspaceInfo>> {
    let pool = Pool::connect_admin().await?;
    let repo = WorkspaceRepository::from_pool(pool).await?;
    let rows = WorkspaceQuery::new(repo)
        .find_workspaces_for_user(user_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(rows
        .into_iter()
        .map(|(id, name)| WorkspaceInfo { id, name })
        .collect())
}

/// Returns the current settings for the given workspace.
pub async fn get_workspace_settings(workspace_id: &str) -> Result<WorkspaceRow> {
    let pool = Pool::connect_admin().await?;
    let repo = WorkspaceRepository::from_pool(pool).await?;
    WorkspaceQuery::new(repo)
        .find_view_by_id(workspace_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .ok_or_else(|| anyhow::anyhow!("workspace not found"))
}

/// Returns all workspace roles for the given workspace.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub async fn list_workspace_roles(workspace_id: &str) -> Result<Vec<WorkspaceRoleRow>> {
    let pool = Pool::connect_admin().await?;
    let repo = WorkspaceRoleRepository::from_pool(pool).await?;
    let all_roles = repo.all().await.map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(all_roles
        .into_iter()
        .filter(|r| r.workspace_id().to_string() == workspace_id)
        .map(|r| {
            WorkspaceRoleRow::new(
                r.id().clone(),
                r.workspace_id().clone(),
                r.name().map(ToOwned::to_owned),
            )
        })
        .collect())
}

/// Records a `WorkspaceSettingsUpdated` event for the given workspace.
pub async fn update_workspace_settings(
    workspace_id: &str,
    name: Option<String>,
    timezone: String,
    date_format: String,
    currency: String,
    week_start: String,
) -> Result<()> {
    let pool = Pool::connect_admin().await?;
    let repo = WorkspaceRepository::from_pool(pool).await?;

    let agg_id: WorkspaceId = workspace_id.parse()?;
    let cmd = WorkspaceCommand::new(repo);
    cmd.update_settings(agg_id, name, timezone, date_format, currency, week_start)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}
