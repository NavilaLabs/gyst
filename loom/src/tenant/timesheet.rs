use anyhow::Result;
use chrono::{DateTime, Utc};
use eventually::aggregate::{
    Root,
    repository::{Getter, Saver},
};
use loom_core::{
    shared::AggregateId,
    tenant::{
        activity::ActivityId,
        timesheet::{Timesheet, TimesheetEvent, TimesheetId, TimesheetRow},
    },
};
use loom_infrastructure_impl::{
    ConnectedTenantPool, tenant::timesheet::repositories::TimesheetRepository,
};

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
    project_id: Option<String>,
    activity_id: Option<String>,
    description: Option<String>,
    billable: bool,
) -> Result<TimesheetRow> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetRepository::from_pool(pool).await?;

    // Enforce: only one running timer per user at a time.
    if repo.running_for_user(user_id).await?.is_some() {
        return Err(crate::error::ValidationError::new(
            "A timer is already running — stop it before starting a new one",
        )
        .into());
    }

    let id = TimesheetId::new();
    let uid: AggregateId = user_id.parse()?;
    let aid: Option<ActivityId> = activity_id.as_deref().map(str::parse).transpose()?;
    let start_time = Utc::now().to_rfc3339();
    let timezone = "UTC".to_string();

    let mut root = Root::<Timesheet>::record_new(
        TimesheetEvent::Started {
            id: id.clone(),
            user_id: uid,
            activity_id: aid,
            start_time: start_time.clone(),
            timezone: timezone.clone(),
        }
        .into(),
    )?;
    if description.is_some() {
        root.record_that(
            TimesheetEvent::Updated {
                description: description.clone(),
            }
            .into(),
        )?;
    }
    repo.save(&mut root).await?;

    Ok(TimesheetRow::new(
        id,
        user_id,
        activity_id,
        start_time,
        None,
        None,
        description,
        timezone,
    ))
}

/// # Errors
///
/// Returns an error if the timesheet cannot be found or saved.
pub async fn reassign(
    workspace_id: &str,
    timesheet_id: &str,
    project_id: String,
    activity_id: String,
) -> Result<()> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetRepository::from_pool(pool).await?;
    let agg_id: TimesheetId = timesheet_id.parse()?;
    let mut root = repo.get(&agg_id).await?;
    let pid: ProjectId = project_id.parse()?;
    let aid: ActivityId = activity_id.parse()?;
    root.record_that(
        TimesheetEvent::Reassigned {
            project_id: pid,
            activity_id: aid,
        }
        .into(),
    )?;
    repo.save(&mut root).await?;
    Ok(())
}

/// # Errors
///
/// Returns an error if the timesheet cannot be found or saved.
pub async fn update(
    workspace_id: &str,
    timesheet_id: &str,
    description: Option<String>,
    billable: bool,
) -> Result<()> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetRepository::from_pool(pool).await?;
    let agg_id: TimesheetId = timesheet_id.parse()?;
    let mut root = repo.get(&agg_id).await?;
    root.record_that(
        TimesheetEvent::Updated {
            description,
            billable,
        }
        .into(),
    )?;
    repo.save(&mut root).await?;
    Ok(())
}

/// # Errors
///
/// Returns an error if the timesheet cannot be found or saved.
pub async fn stop(workspace_id: &str, timesheet_id: &str) -> Result<()> {
    let pool = super::tenant_pool(workspace_id).await?;
    let ts_repo = TimesheetRepository::from_pool(pool.clone()).await?;

    let agg_id: TimesheetId = timesheet_id.parse()?;
    let mut root = ts_repo.get(&agg_id).await?;

    let end_time = Utc::now();
    let end_rfc = end_time.to_rfc3339();
    let duration = chrono::DateTime::parse_from_rfc3339(root.start_time())
        .ok()
        .map_or(0, |start| {
            #[allow(clippy::cast_possible_truncation)]
            let secs = (end_time - start.with_timezone(&Utc)).num_seconds() as i32;
            secs
        });

    // Look up the applicable rate (only possible when project/activity are assigned)
    let project_id = root.project_id().map(std::string::ToString::to_string);
    let activity_id = root.activity_id().map(std::string::ToString::to_string);
    let (hourly_rate, internal_rate) = match (&project_id, &activity_id) {
        (Some(pid), Some(aid)) => resolve_rate(&pool, pid, aid).await,
        _ => (None, None),
    };
    let rate = hourly_rate.map(|hr| hr * i64::from(duration) / 3600);

    root.record_that(
        TimesheetEvent::Stopped {
            end_time: end_rfc,
            duration,
            hourly_rate,
            fixed_rate: None,
            internal_rate,
            rate,
        }
        .into(),
    )?;
    ts_repo.save(&mut root).await?;
    Ok(())
}

/// # Errors
///
/// Returns an error if the timesheet cannot be found or saved.
pub async fn export(workspace_id: &str, timesheet_id: &str) -> Result<()> {
    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetRepository::from_pool(pool).await?;
    let agg_id: TimesheetId = timesheet_id.parse()?;
    let mut root = repo.get(&agg_id).await?;
    root.record_that(TimesheetEvent::Exported.into())?;
    repo.save(&mut root).await?;
    Ok(())
}

/// Create a completed timesheet from explicit start and end times.
///
/// Used for manual ("after the fact") time entry.  Times are accepted as either
/// RFC-3339 strings or HTML `datetime-local` values (`YYYY-MM-DDTHH:MM`), both
/// interpreted as UTC.
///
/// # Errors
///
/// Returns an error if the times are invalid, out of order, or the timesheet cannot be saved.
#[allow(clippy::too_many_arguments)]
pub async fn create_manual(
    workspace_id: &str,
    user_id: &str,
    project_id: Option<String>,
    activity_id: Option<String>,
    start_time: String,
    end_time: String,
    description: Option<String>,
    billable: bool,
) -> Result<TimesheetRow> {
    let start_dt = parse_datetime_utc(&start_time)?;
    let end_dt = parse_datetime_utc(&end_time)?;
    if end_dt <= start_dt {
        return Err(crate::error::ValidationError::new("End time must be after start time").into());
    }
    #[allow(clippy::cast_possible_truncation)]
    let duration = (end_dt - start_dt).num_seconds() as i32;
    let start_rfc = start_dt.to_rfc3339();
    let end_rfc = end_dt.to_rfc3339();

    let pool = super::tenant_pool(workspace_id).await?;
    let repo = TimesheetRepository::from_pool(pool.clone()).await?;

    let id = TimesheetId::new();
    let uid: AggregateId = user_id.parse()?;
    let pid: Option<ProjectId> = project_id.as_deref().map(str::parse).transpose()?;
    let aid: Option<ActivityId> = activity_id.as_deref().map(str::parse).transpose()?;
    let pid_str = pid.as_ref().map(ToString::to_string);
    let aid_str = aid.as_ref().map(ToString::to_string);

    let (hourly_rate, internal_rate) = match (&pid_str, &aid_str) {
        (Some(p), Some(a)) => resolve_rate(&pool, p, a).await,
        _ => (None, None),
    };
    let rate = hourly_rate.map(|hr| hr * i64::from(duration) / 3600);

    let mut root = Root::<Timesheet>::record_new(
        TimesheetEvent::Started {
            id: id.clone(),
            user_id: uid,
            project_id: pid,
            activity_id: aid,
            start_time: start_rfc.clone(),
            timezone: "UTC".to_string(),
            billable,
        }
        .into(),
    )?;
    root.record_that(
        TimesheetEvent::Stopped {
            end_time: end_rfc.clone(),
            duration,
            hourly_rate,
            fixed_rate: None,
            internal_rate,
            rate,
        }
        .into(),
    )?;
    if let Some(ref desc) = description
        && !desc.is_empty()
    {
        root.record_that(
            TimesheetEvent::Updated {
                description: Some(desc.clone()),
                billable,
            }
            .into(),
        )?;
    }
    repo.save(&mut root).await?;

    Ok(TimesheetRow {
        id: id.to_string(),
        user_id: user_id.to_string(),
        project_id: pid_str,
        activity_id: aid_str,
        start_time: start_rfc,
        end_time: Some(end_rfc),
        duration: Some(duration),
        description,
        timezone: "UTC".to_string(),
        billable,
        exported: false,
        hourly_rate,
        fixed_rate: None,
        internal_rate,
        rate,
    })
}

/// Edit the start time (and optionally the end time) of any timesheet.
///
/// For a stopped timesheet both `start_time` and `end_time` must be supplied.
/// For a running timer supply only `start_time`; `end_time` must be `None`.
/// Times are accepted as RFC-3339 or `datetime-local` (`YYYY-MM-DDTHH:MM`) UTC.
///
/// # Errors
///
/// Returns an error if the times are invalid, out of order, or the timesheet cannot be saved.
pub async fn update_time(
    workspace_id: &str,
    timesheet_id: &str,
    start_time: String,
    end_time: Option<String>,
) -> Result<()> {
    let start_dt = parse_datetime_utc(&start_time)?;
    let (end_rfc, duration) = if let Some(ref et) = end_time {
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
    let mut root = repo.get(&agg_id).await?;
    root.record_that(
        TimesheetEvent::TimeUpdated {
            start_time: start_dt.to_rfc3339(),
            end_time: end_rfc,
            duration,
        }
        .into(),
    )?;
    repo.save(&mut root).await?;
    Ok(())
}

/// Parse a datetime string (RFC-3339 or HTML `datetime-local`) as UTC.
fn parse_datetime_utc(s: &str) -> Result<DateTime<Utc>> {
    // Try RFC-3339 / ISO-8601 with offset first.
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }
    // HTML datetime-local: "YYYY-MM-DDTHH:MM" (16) or "YYYY-MM-DDTHH:MM:SS" (19) — treat as UTC.
    let with_z = match s.len() {
        16 => format!("{s}:00Z"),
        // 19 => format!("{s}Z"),
        _ => format!("{s}Z"),
    };
    DateTime::parse_from_rfc3339(&with_z)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| anyhow::anyhow!("Invalid date/time '{s}': {e}"))
}

/// Resolve the billing rate for a timesheet: project rate first, activity rate as fallback.
/// Returns `(hourly_rate_cents, internal_rate_cents)`.
async fn resolve_rate(
    pool: &ConnectedTenantPool,
    project_id: &str,
    activity_id: &str,
) -> (Option<i64>, Option<i64>) {
    if let Ok(repo) = ProjectRateRepository::from_pool(pool.clone()).await
        && let Ok(Some(row)) = repo.default_for_project(project_id).await
    {
        return (Some(row.hourly_rate), row.internal_rate);
    }
    if let Ok(repo) = ActivityRateRepository::from_pool(pool.clone()).await
        && let Ok(Some(row)) = repo.default_for_activity(activity_id).await
    {
        return (Some(row.hourly_rate), row.internal_rate);
    }
    (None, None)
}
