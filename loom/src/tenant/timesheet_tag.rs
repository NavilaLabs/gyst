use anyhow::Result;
use eventually::aggregate::{
    Root,
    repository::{Getter, Saver},
};
use loom_core::tenant::{
    timesheet::TimesheetId,
    timesheet_tag::{CreateTagInput, RenameTagInput, Tag, TagEvent, TagId},
};
use loom_infrastructure_impl::tenant::timesheet_tag::repositories::{TagRepository, TagRow};

pub async fn list(workspace_id: &str) -> Result<Vec<TagRow>> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TagRepository::from_pool(pool).await?;
    Ok(repo.all().await?)
}

pub async fn list_for_timesheet(workspace_id: &str, timesheet_id: &str) -> Result<Vec<TagRow>> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TagRepository::from_pool(pool).await?;
    Ok(repo.for_timesheet(timesheet_id).await?)
}

pub async fn create(workspace_id: &str, name: String) -> Result<TagRow> {
    crate::error::validate(CreateTagInput { name: name.clone() })?;

    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TagRepository::from_pool(pool).await?;
    let id = TagId::new();
    let mut root = Root::<Tag>::record_new(
        TagEvent::Created {
            id: id.clone(),
            name: name.clone(),
        }
        .into(),
    )?;
    repo.save(&mut root).await?;
    Ok(TagRow {
        id: id.to_string(),
        name,
    })
}

pub async fn rename(workspace_id: &str, id: &str, name: String) -> Result<()> {
    crate::error::validate(RenameTagInput { name: name.clone() })?;

    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TagRepository::from_pool(pool).await?;
    let agg_id: TagId = id.parse()?;
    let mut root = repo.get(&agg_id).await?;
    root.record_that(TagEvent::Renamed { name }.into())?;
    repo.save(&mut root).await?;
    Ok(())
}

pub async fn tag_timesheet(workspace_id: &str, tag_id: &str, timesheet_id: &str) -> Result<()> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TagRepository::from_pool(pool).await?;
    let agg_id: TagId = tag_id.parse()?;
    let ts_id: TimesheetId = timesheet_id.parse()?;
    let mut root = repo.get(&agg_id).await?;
    root.record_that(
        TagEvent::TimesheetTagged {
            timesheet_id: ts_id,
        }
        .into(),
    )?;
    repo.save(&mut root).await?;
    Ok(())
}

pub async fn untag_timesheet(workspace_id: &str, tag_id: &str, timesheet_id: &str) -> Result<()> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TagRepository::from_pool(pool).await?;
    let agg_id: TagId = tag_id.parse()?;
    let ts_id: TimesheetId = timesheet_id.parse()?;
    let mut root = repo.get(&agg_id).await?;
    root.record_that(
        TagEvent::TimesheetUntagged {
            timesheet_id: ts_id,
        }
        .into(),
    )?;
    repo.save(&mut root).await?;
    Ok(())
}
