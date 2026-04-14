use anyhow::Result;
use eventually::aggregate::repository::{Getter, Saver};
use loom_core::tenant::activity::{
    ActivityCommand, ActivityId, ActivityRow, CreateActivityInput, UpdateActivityInput,
};
use loom_infrastructure_impl::tenant::activity::repositories::ActivityRepository;

/// List all activities for a workspace.
pub async fn list(workspace_id: &str) -> Result<Vec<ActivityRow>> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = ActivityRepository::from_pool(pool).await?;
    Ok(repo.all().await?)
}

/// Create a new activity, returning the saved view.
pub async fn create(
    workspace_id: &str,
    name: String,
    comment: Option<String>,
) -> Result<ActivityRow> {
    crate::error::validate(CreateActivityInput { name: name.clone() })?;

    let pool = super::tenant_pool(workspace_id).await?;
    let repo = ActivityRepository::from_pool(pool).await?;

    let id = ActivityId::new();
    let mut cmd = ActivityCommand::create(id.clone(), name.clone(), comment.clone())?;
    repo.save(&mut cmd).await?;

    Ok(ActivityRow::new(id, name, comment))
}

/// Update an existing activity's name and optional comment.
pub async fn update(
    workspace_id: &str,
    id: &str,
    name: String,
    comment: Option<String>,
) -> Result<()> {
    crate::error::validate(UpdateActivityInput { name: name.clone() })?;

    let pool = super::tenant_pool(workspace_id).await?;
    let repo = ActivityRepository::from_pool(pool).await?;

    let agg_id: ActivityId = id.parse()?;
    let root = repo.get(&agg_id).await?;
    let mut cmd: ActivityCommand = root.into();
    cmd.update(name, comment)?;
    repo.save(&mut cmd).await?;

    Ok(())
}
