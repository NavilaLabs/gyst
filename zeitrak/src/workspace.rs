use anyhow::Result;
use zeitrak_core::admin::workspace::{WorkspaceCommand, WorkspaceCommandTrait, WorkspaceId, WorkspaceRow};
use zeitrak_infrastructure_impl::{Pool, admin::workspace::repositories::WorkspaceRepository};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub id: String,
    pub name: Option<String>,
}

/// Returns all workspaces the given user is a member of.
pub async fn list_user_workspaces(user_id: &str) -> Result<Vec<WorkspaceInfo>> {
    let pool = Pool::connect_admin().await?;
    let repo = WorkspaceRepository::from_pool(pool).await?;
    let rows = repo.find_workspaces_for_user(user_id).await?;
    Ok(rows
        .into_iter()
        .map(|(id, name)| WorkspaceInfo { id, name })
        .collect())
}

/// Returns the current settings for the given workspace.
pub async fn get_workspace_settings(workspace_id: &str) -> Result<WorkspaceRow> {
    let pool = Pool::connect_admin().await?;
    let repo = WorkspaceRepository::from_pool(pool).await?;
    repo.find_view_by_id(workspace_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("workspace not found"))
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
    let mut cmd = WorkspaceCommand::new(repo);
    cmd.update_settings(agg_id, name, timezone, date_format, currency, week_start)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}
