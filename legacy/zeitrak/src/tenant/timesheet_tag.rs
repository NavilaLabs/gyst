use std::collections::HashMap;

use anyhow::Result;
use zeitrak_core::tenant::{
    timesheet::TimesheetId,
    timesheet_tag::{
        CreateTimesheetTagInput, RenameTimesheetTagInput, TimesheetTagHandler,
        TimesheetTagHandlerTrait, TimesheetTagId, TimesheetTagQuery, TimesheetTagQueryTrait,
        TimesheetTagRow,
    },
};
use zeitrak_infrastructure_impl::tenant::timesheet_tag::repositories::TimesheetTagRepository;

pub async fn list(workspace_id: &str) -> Result<Vec<TimesheetTagRow>> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetTagRepository::from_pool(pool).await?;
    TimesheetTagQuery::new(repo)
        .list_all()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

pub async fn list_for_timesheet(
    workspace_id: &str,
    timesheet_id: &str,
) -> Result<Vec<TimesheetTagRow>> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetTagRepository::from_pool(pool).await?;
    TimesheetTagQuery::new(repo)
        .for_timesheet(timesheet_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

pub async fn create(workspace_id: &str, name: String) -> Result<TimesheetTagRow> {
    crate::error::validate(&CreateTimesheetTagInput { name: name.clone() })?;

    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetTagRepository::from_pool(pool).await?;
    let id = TimesheetTagId::new();
    TimesheetTagHandler::new(repo)
        .create(id, name)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

pub async fn rename(workspace_id: &str, id: &str, name: String) -> Result<()> {
    crate::error::validate(&RenameTimesheetTagInput { name: name.clone() })?;

    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetTagRepository::from_pool(pool).await?;
    let agg_id: TimesheetTagId = id.parse()?;
    TimesheetTagHandler::new(repo)
        .rename(agg_id, name)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

pub async fn tag_timesheet(workspace_id: &str, tag_id: &str, timesheet_id: &str) -> Result<()> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetTagRepository::from_pool(pool).await?;
    let agg_id: TimesheetTagId = tag_id.parse()?;
    let ts_id: TimesheetId = timesheet_id.parse()?;
    TimesheetTagHandler::new(repo)
        .tag_timesheet(agg_id, ts_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

pub async fn untag_timesheet(workspace_id: &str, tag_id: &str, timesheet_id: &str) -> Result<()> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetTagRepository::from_pool(pool).await?;
    let agg_id: TimesheetTagId = tag_id.parse()?;
    let ts_id: TimesheetId = timesheet_id.parse()?;
    TimesheetTagHandler::new(repo)
        .untag_timesheet(agg_id, ts_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

pub async fn delete(workspace_id: &str, id: &str) -> Result<()> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetTagRepository::from_pool(pool).await?;
    let agg_id: TimesheetTagId = id.parse()?;
    TimesheetTagHandler::new(repo)
        .delete(agg_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Returns all tag assignments for the given timesheet IDs, grouped by `timesheet_id`.
pub async fn for_timesheets_batch(
    workspace_id: &str,
    timesheet_ids: &[&str],
) -> Result<HashMap<String, Vec<TimesheetTagRow>>> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetTagRepository::from_pool(pool).await?;
    TimesheetTagQuery::new(repo)
        .for_timesheets_batch(timesheet_ids)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}
