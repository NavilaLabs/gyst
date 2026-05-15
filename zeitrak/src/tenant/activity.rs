use anyhow::Result;
use zeitrak_core::tenant::activity::{
    ActivityHandler, ActivityHandlerTrait, ActivityId, ActivityQuery, ActivityQueryTrait,
    ActivityRow, CreateActivityInput, UpdateActivityInput,
};
use zeitrak_infrastructure_impl::tenant::activity::repositories::ActivityRepository;

/// List all activities for a workspace.
pub async fn list(workspace_id: &str) -> Result<Vec<ActivityRow>> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = ActivityRepository::from_pool(pool).await?;
    ActivityQuery::new(repo)
        .list_all()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Create a new activity, returning the saved view.
pub async fn create(
    workspace_id: &str,
    name: String,
    color: String,
    comment: Option<String>,
) -> Result<ActivityRow> {
    crate::error::validate(&CreateActivityInput {
        name: name.clone(),
        color: color.clone(),
    })?;

    let pool = super::tenant_pool(workspace_id).await?;
    let repo = ActivityRepository::from_pool(pool).await?;
    let id = ActivityId::new();
    ActivityHandler::new(repo)
        .create(id, name, color, comment)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Soft-delete an activity (excluded from future queries).
pub async fn delete(workspace_id: &str, id: &str) -> Result<()> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = ActivityRepository::from_pool(pool).await?;
    let agg_id: ActivityId = id.parse()?;
    ActivityHandler::new(repo)
        .delete(agg_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Update an existing activity's name, color, and optional comment.
pub async fn update(
    workspace_id: &str,
    id: &str,
    name: String,
    color: String,
    comment: Option<String>,
) -> Result<()> {
    crate::error::validate(&UpdateActivityInput {
        name: name.clone(),
        color: color.clone(),
    })?;

    let pool = super::tenant_pool(workspace_id).await?;
    let repo = ActivityRepository::from_pool(pool).await?;
    let agg_id: ActivityId = id.parse()?;
    ActivityHandler::new(repo)
        .update(agg_id, name, color, comment)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}
