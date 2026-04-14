use anyhow::Result;
use eventually::aggregate::{
    Root,
    repository::{Getter, Saver},
};
use loom_core::tenant::activity::{
    Activity, ActivityEvent, ActivityId, ActivityRow, CreateActivityInput, UpdateActivityInput,
};
use loom_infrastructure_impl::tenant::activity::repositories::ActivityRepository;

pub async fn list(workspace_id: &str) -> Result<Vec<ActivityRow>> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = ActivityRepository::from_pool(pool).await?;
    Ok(repo.all().await?)
}

pub async fn create(
    workspace_id: &str,
    name: String,
    comment: Option<String>,
) -> Result<ActivityRow> {
    crate::error::validate(CreateActivityInput { name: name.clone() })?;

    let pool = super::tenant_pool(workspace_id).await?;
    let repo = ActivityRepository::from_pool(pool).await?;
    let id = ActivityId::new();
    let mut root = Root::<Activity>::record_new(
        ActivityEvent::Created {
            id: id.clone(),
            name: name.clone(),
            comment: comment.clone(),
        }
        .into(),
    )?;
    repo.save(&mut root).await?;
    Ok(ActivityRow::new(id, name, comment))
}

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
    let mut root = repo.get(&agg_id).await?;
    root.record_that(ActivityEvent::Updated { name, comment }.into())?;
    repo.save(&mut root).await?;
    Ok(())
}
