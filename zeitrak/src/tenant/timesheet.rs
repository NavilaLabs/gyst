use anyhow::Result;
use chrono::{DateTime, Utc};
use eventually::aggregate::repository::{Getter, Saver};
use zeitrak_core::{
    shared::AggregateId,
    tenant::{
        activity::ActivityId,
        timesheet::{TimesheetCommand, TimesheetCommandTrait, TimesheetId, TimesheetRow},
    },
};
use zeitrak_infrastructure_impl::tenant::timesheet::repositories::TimesheetRepository;

pub async fn recent(workspace_id: &str, user_id: &str) -> Result<Vec<TimesheetRow>> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetRepository::from_pool(pool).await?;
    Ok(repo.recent_for_user(user_id).await?)
}

pub async fn running(workspace_id: &str, user_id: &str) -> Result<Option<TimesheetRow>> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetRepository::from_pool(pool).await?;
    Ok(repo.running_for_user(user_id).await?)
}

pub async fn start(
    workspace_id: &str,
    user_id: &str,
    activity_id: Option<&str>,
    description: Option<String>,
) -> Result<TimesheetRow> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetRepository::from_pool(pool).await?;

    if repo.running_for_user(user_id).await?.is_some() {
        return Err(crate::error::ValidationError::new(
            "A timer is already running — stop it before starting a new one",
        )
        .into());
    }

    let id = TimesheetId::new();
    let uid: AggregateId = user_id.parse()?;
    let aid: Option<ActivityId> = activity_id.map(str::parse).transpose()?;
    let start_time = Utc::now().to_rfc3339();
    let timezone = "UTC".to_string();

    let mut cmd = TimesheetCommand::start(
        id.clone(),
        uid.clone(),
        aid.clone(),
        start_time.clone(),
        timezone.clone(),
    )?;
    if let Some(ref desc) = description {
        cmd.update(Some(desc.clone()))?;
    }
    repo.save(&mut cmd).await?;

    Ok(TimesheetRow::new(
        id,
        uid,
        aid,
        start_time,
        None,
        None,
        description,
        timezone,
    ))
}

pub async fn stop(workspace_id: &str, timesheet_id: &str) -> Result<()> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetRepository::from_pool(pool).await?;

    let agg_id: TimesheetId = timesheet_id.parse()?;
    let root = repo.get(&agg_id).await?;
    let mut cmd: TimesheetCommand = root.into();

    let end_time = Utc::now();
    let end_rfc = end_time.to_rfc3339();
    #[allow(clippy::cast_possible_truncation)]
    let duration = DateTime::parse_from_rfc3339(cmd.start_time())
        .ok()
        .map_or(0, |start| {
            (end_time - start.with_timezone(&Utc)).num_seconds() as i32
        });

    cmd.stop(end_rfc, duration)?;
    repo.save(&mut cmd).await?;
    Ok(())
}

pub async fn update(
    workspace_id: &str,
    timesheet_id: &str,
    description: Option<String>,
) -> Result<()> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetRepository::from_pool(pool).await?;
    let agg_id: TimesheetId = timesheet_id.parse()?;
    let root = repo.get(&agg_id).await?;
    let mut cmd: TimesheetCommand = root.into();
    cmd.update(description)?;
    repo.save(&mut cmd).await?;
    Ok(())
}

pub async fn reassign(workspace_id: &str, timesheet_id: &str, activity_id: &str) -> Result<()> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetRepository::from_pool(pool).await?;
    let agg_id: TimesheetId = timesheet_id.parse()?;
    let root = repo.get(&agg_id).await?;
    let mut cmd: TimesheetCommand = root.into();
    let aid: ActivityId = activity_id.parse()?;
    cmd.reassign(aid)?;
    repo.save(&mut cmd).await?;
    Ok(())
}

/// Create a completed timesheet from explicit start and end times (manual entry).
///
/// Times are accepted as either RFC-3339 strings or HTML `datetime-local` values
/// (`YYYY-MM-DDTHH:MM`), both interpreted as UTC.
pub async fn create_manual(
    workspace_id: &str,
    user_id: &str,
    activity_id: Option<&str>,
    start_time: &str,
    end_time: &str,
    description: Option<String>,
) -> Result<TimesheetRow> {
    let start_dt = parse_datetime_utc(start_time)?;
    let end_dt = parse_datetime_utc(end_time)?;
    if end_dt <= start_dt {
        return Err(crate::error::ValidationError::new("End time must be after start time").into());
    }
    #[allow(clippy::cast_possible_truncation)]
    let duration = (end_dt - start_dt).num_seconds() as i32;
    let start_rfc = start_dt.to_rfc3339();
    let end_rfc = end_dt.to_rfc3339();

    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetRepository::from_pool(pool).await?;

    let id = TimesheetId::new();
    let uid: AggregateId = user_id.parse()?;
    let aid: Option<ActivityId> = activity_id.map(str::parse).transpose()?;
    let timezone = "UTC".to_string();

    let mut cmd = TimesheetCommand::start(
        id.clone(),
        uid.clone(),
        aid.clone(),
        start_rfc.clone(),
        timezone.clone(),
    )?;
    cmd.stop(end_rfc.clone(), duration)?;
    if let Some(ref desc) = description {
        cmd.update(Some(desc.clone()))?;
    }
    repo.save(&mut cmd).await?;

    Ok(TimesheetRow::new(
        id,
        uid,
        aid,
        start_rfc,
        Some(end_rfc),
        Some(duration),
        description,
        timezone,
    ))
}

pub async fn cancel(workspace_id: &str, timesheet_id: &str) -> Result<()> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetRepository::from_pool(pool).await?;
    let agg_id: TimesheetId = timesheet_id.parse()?;
    let root = repo.get(&agg_id).await?;
    let mut cmd: TimesheetCommand = root.into();
    cmd.cancel()?;
    repo.save(&mut cmd).await?;
    Ok(())
}

pub async fn update_time(
    workspace_id: &str,
    timesheet_id: &str,
    start_time: &str,
    end_time: Option<&str>,
) -> Result<()> {
    let start_dt = parse_datetime_utc(start_time)?;
    let (end_rfc, duration) = if let Some(et) = end_time {
        let end_dt = parse_datetime_utc(et)?;
        if end_dt <= start_dt {
            return Err(
                crate::error::ValidationError::new("End time must be after start time").into(),
            );
        }
        #[allow(clippy::cast_possible_truncation)]
        let dur = (end_dt - start_dt).num_seconds() as i32;
        (Some(end_dt.to_rfc3339()), Some(dur))
    } else {
        (None, None)
    };

    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetRepository::from_pool(pool).await?;
    let agg_id: TimesheetId = timesheet_id.parse()?;
    let root = repo.get(&agg_id).await?;
    let mut cmd: TimesheetCommand = root.into();
    cmd.update_time(start_dt.to_rfc3339(), end_rfc, duration)?;
    repo.save(&mut cmd).await?;
    Ok(())
}

fn parse_datetime_utc(s: &str) -> Result<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }
    let with_z = match s.len() {
        16 => format!("{s}:00Z"),
        _ => format!("{s}Z"),
    };
    DateTime::parse_from_rfc3339(&with_z)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| anyhow::anyhow!("Invalid date/time '{s}': {e}"))
}
