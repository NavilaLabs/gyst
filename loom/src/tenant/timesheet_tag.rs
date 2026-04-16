use std::collections::HashMap;

use anyhow::Result;
use eventually::aggregate::repository::{Getter, Saver};
use loom_core::tenant::{
    timesheet::TimesheetId,
    timesheet_tag::{
        CreateTimesheetTagInput, RenameTimesheetTagInput, TimesheetTagCommand, TimesheetTagId,
        TimesheetTagRow,
    },
};
use loom_infrastructure_impl::tenant::timesheet_tag::repositories::TimesheetTagRepository;

pub async fn list(workspace_id: &str) -> Result<Vec<TimesheetTagRow>> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetTagRepository::from_pool(pool).await?;
    Ok(repo.all().await?)
}

pub async fn list_for_timesheet(
    workspace_id: &str,
    timesheet_id: &str,
) -> Result<Vec<TimesheetTagRow>> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetTagRepository::from_pool(pool).await?;
    Ok(repo.for_timesheet(timesheet_id).await?)
}

pub async fn create(workspace_id: &str, name: String) -> Result<TimesheetTagRow> {
    crate::error::validate(CreateTimesheetTagInput { name: name.clone() })?;

    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetTagRepository::from_pool(pool).await?;
    let id = TimesheetTagId::new();
    let mut cmd = TimesheetTagCommand::create(id.clone(), name.clone())?;
    repo.save(&mut cmd).await?;
    Ok(TimesheetTagRow::new(id, name))
}

pub async fn rename(workspace_id: &str, id: &str, name: String) -> Result<()> {
    crate::error::validate(RenameTimesheetTagInput { name: name.clone() })?;

    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetTagRepository::from_pool(pool).await?;
    let agg_id: TimesheetTagId = id.parse()?;
    let root = repo.get(&agg_id).await?;
    let mut cmd: TimesheetTagCommand = root.into();
    cmd.rename(name)?;
    repo.save(&mut cmd).await?;
    Ok(())
}

pub async fn tag_timesheet(workspace_id: &str, tag_id: &str, timesheet_id: &str) -> Result<()> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetTagRepository::from_pool(pool).await?;
    let agg_id: TimesheetTagId = tag_id.parse()?;
    let ts_id: TimesheetId = timesheet_id.parse()?;
    let root = repo.get(&agg_id).await?;
    let mut cmd: TimesheetTagCommand = root.into();
    cmd.tag_timesheet(ts_id)?;
    repo.save(&mut cmd).await?;
    Ok(())
}

pub async fn untag_timesheet(workspace_id: &str, tag_id: &str, timesheet_id: &str) -> Result<()> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetTagRepository::from_pool(pool).await?;
    let agg_id: TimesheetTagId = tag_id.parse()?;
    let ts_id: TimesheetId = timesheet_id.parse()?;
    let root = repo.get(&agg_id).await?;
    let mut cmd: TimesheetTagCommand = root.into();
    cmd.untag_timesheet(ts_id)?;
    repo.save(&mut cmd).await?;
    Ok(())
}

pub async fn delete(workspace_id: &str, id: &str) -> Result<()> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetTagRepository::from_pool(pool).await?;
    let agg_id: TimesheetTagId = id.parse()?;
    let root = repo.get(&agg_id).await?;
    let mut cmd: TimesheetTagCommand = root.into();
    cmd.delete()?;
    repo.save(&mut cmd).await?;
    Ok(())
}

/// Returns all tag assignments for the given timesheet IDs, grouped by `timesheet_id`.
pub async fn for_timesheets_batch(
    workspace_id: &str,
    timesheet_ids: &[&str],
) -> Result<HashMap<String, Vec<TimesheetTagRow>>> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetTagRepository::from_pool(pool).await?;
    Ok(repo.for_timesheets_batch(timesheet_ids).await?)
}
