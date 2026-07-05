use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::timesheet_tag::TimesheetsTagDto;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimesheetDto {
    pub id: String,
    pub user_id: String,
    pub activity_id: Option<String>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration: Option<i32>,
    pub description: Option<String>,
    pub timezone: String,
    pub tags: Vec<TimesheetsTagDto>,
    /// Display name of the member who created this timesheet.
    /// Populated only when the caller has `timesheet.read_all` access.
    pub member_name: Option<String>,
}

/// A page of timesheet results with server-side pagination metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimesheetPageDto {
    pub items: Vec<TimesheetDto>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    /// `true` when the caller has `timesheet.read_all` and may use the `member_id` filter.
    pub can_filter_members: bool,
}

/// One time bucket in the dashboard bar chart.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyChartPoint {
    /// Human-readable label for the bucket (e.g. "Mon", "Jun 1", "Jan").
    pub label: String,
    /// `(activity_id, activity_name, color, hours)` segments for this bucket.
    pub segments: Vec<(String, String, String, f32)>,
}

/// One slice in the dashboard activity-mix donut.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityMixPoint {
    pub activity_id: String,
    pub activity_name: String,
    pub color: String,
    pub hours: f32,
    pub percentage: f32,
}

/// Chart aggregation period for `dashboard_stats`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DashboardPeriod {
    Week,
    Month,
    Year,
}

/// Pre-aggregated stats returned by `dashboard_stats`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashboardStatsDto {
    pub today_hours: f32,
    pub week_hours: f32,
    pub streak: u32,
    pub chart_bars: Vec<DailyChartPoint>,
    pub activity_mix: Vec<ActivityMixPoint>,
    /// `true` when the caller has `timesheet.read_all` and may use the `member_id` filter.
    pub can_filter_members: bool,
}

/// One row in the monthly overview table on the dashboard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonthlyOverviewRow {
    pub year: i32,
    /// Calendar month (1–12).
    pub month: u32,
    pub total_hours: f32,
    pub activity_mix: Vec<ActivityMixPoint>,
}

#[post("/api/timesheets/recent")]
pub async fn list_timesheets(
    page: u32,
    member_id: Option<String>,
) -> Result<TimesheetPageDto, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _list_timesheets(page, member_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (page, member_id);
        Ok(TimesheetPageDto {
            items: vec![],
            total: 0,
            page: 0,
            page_size: 20,
            can_filter_members: false,
        })
    }
}

#[post("/api/timesheets/dashboard-stats")]
pub async fn dashboard_stats(
    member_id: Option<String>,
    period: DashboardPeriod,
) -> Result<DashboardStatsDto, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _dashboard_stats(member_id, period).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (member_id, period);
        Ok(DashboardStatsDto {
            today_hours: 0.0,
            week_hours: 0.0,
            streak: 0,
            chart_bars: vec![],
            activity_mix: vec![],
            can_filter_members: false,
        })
    }
}

#[post("/api/timesheets/monthly-overview")]
pub async fn monthly_overview(
    member_id: Option<String>,
) -> Result<Vec<MonthlyOverviewRow>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _monthly_overview(member_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = member_id;
        Ok(vec![])
    }
}

#[get("/api/timesheets/running")]
pub async fn running_timesheet() -> Result<Option<TimesheetDto>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _running_timesheet().await
    }
    #[cfg(not(feature = "server"))]
    {
        Ok(None)
    }
}

#[post("/api/timesheets/start")]
pub async fn start_timesheet(
    activity_id: Option<String>,
    description: Option<String>,
) -> Result<TimesheetDto, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _start_timesheet(activity_id, description).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (activity_id, description);
        Err(ServerFnError::ServerError {
            message: "server only".into(),
            code: 500,
            details: None,
        })
    }
}

#[post("/api/timesheets/reassign")]
pub async fn reassign_timesheet(
    timesheet_id: String,
    activity_id: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _reassign_timesheet(timesheet_id, activity_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (timesheet_id, activity_id);
        Ok(())
    }
}

#[post("/api/timesheets/update")]
pub async fn update_timesheet(
    timesheet_id: String,
    description: Option<String>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _update_timesheet(timesheet_id, description).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (timesheet_id, description);
        Ok(())
    }
}

#[post("/api/timesheets/create-manual")]
pub async fn create_timesheet_manual(
    activity_id: Option<String>,
    start_time: String,
    end_time: String,
    description: Option<String>,
) -> Result<TimesheetDto, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _create_timesheet_manual(activity_id, start_time, end_time, description).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (activity_id, start_time, end_time, description);
        Err(ServerFnError::ServerError {
            message: "server only".into(),
            code: 500,
            details: None,
        })
    }
}

#[post("/api/timesheets/update-time")]
pub async fn update_timesheet_time(
    timesheet_id: String,
    start_time: String,
    end_time: Option<String>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _update_timesheet_time(timesheet_id, start_time, end_time).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (timesheet_id, start_time, end_time);
        Ok(())
    }
}

#[post("/api/timesheets/stop")]
pub async fn stop_timesheet(timesheet_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _stop_timesheet(timesheet_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = timesheet_id;
        Ok(())
    }
}

#[post("/api/timesheets/cancel")]
pub async fn cancel_timesheet(timesheet_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _cancel_timesheet(timesheet_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = timesheet_id;
        Ok(())
    }
}

/// Per-activity aggregated stat for the timeline metrics bar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityStatDto {
    pub activity_id: String,
    pub activity_name: String,
    pub color: String,
    pub total_seconds: i64,
    /// Percentage of total tracked time (0–100).
    pub percentage: f32,
}

/// Aggregated stats returned by `get_timeline_stats` for the metrics bar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineStatsDto {
    pub total_seconds: i64,
    pub by_activity: Vec<ActivityStatDto>,
    pub can_filter_members: bool,
}

#[post("/api/timesheets/timeline")]
pub async fn get_timeline_entries(
    page: u32,
    from: Option<String>,
    to: Option<String>,
    member_id: Option<String>,
) -> Result<TimesheetPageDto, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _get_timeline_entries(page, from, to, member_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (page, from, to, member_id);
        Ok(TimesheetPageDto {
            items: vec![],
            total: 0,
            page: 0,
            page_size: 50,
            can_filter_members: false,
        })
    }
}

#[post("/api/timesheets/timeline-stats")]
pub async fn get_timeline_stats(
    from: Option<String>,
    to: Option<String>,
    member_id: Option<String>,
) -> Result<TimelineStatsDto, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _get_timeline_stats(from, to, member_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (from, to, member_id);
        Ok(TimelineStatsDto {
            total_seconds: 0,
            by_activity: vec![],
            can_filter_members: false,
        })
    }
}

#[cfg(feature = "server")]
fn row_to_dto(
    r: zeitrak::core::tenant::timesheet::TimesheetRow,
    tags: Vec<TimesheetsTagDto>,
    member_name: Option<String>,
) -> TimesheetDto {
    TimesheetDto {
        id: r.id().to_string(),
        user_id: r.user_id().to_string(),
        activity_id: r.activity_id().map(|id| id.to_string()),
        start_time: r.start_time().to_string(),
        end_time: r.end_time().map(String::from),
        duration: r.duration(),
        description: r.description().map(String::from),
        timezone: r.timezone().to_string(),
        tags,
        member_name,
    }
}

#[cfg(feature = "server")]
async fn _list_timesheets(
    page: u32,
    member_id: Option<String>,
) -> Result<TimesheetPageDto, ServerFnError> {
    use std::collections::HashMap;

    use crate::session;
    use zeitrak::authentication::CurrentUser;
    use zeitrak::authorization::AuthorizationService;
    use zeitrak::core::permissions;

    const PAGE_SIZE: u32 = 20;

    let (user, workspace_id) = session::session_workspace().await?;

    let current_user = CurrentUser {
        id: user.id.clone(),
        email: user.email.clone(),
    };
    let is_admin = AuthorizationService::is_admin(&current_user.id)
        .await
        .map_err(session::internal)?;
    let can_read_all = is_admin
        || AuthorizationService::has_permission(
            &current_user.id,
            &workspace_id,
            permissions::TIMESHEET_READ_ALL,
        )
        .await
        .map_err(session::internal)?;

    // Security: member_id filter is only applied when the caller has read_all.
    let effective_member_id = if can_read_all {
        member_id.as_deref()
    } else {
        None
    };

    let (rows, total) = if can_read_all {
        zeitrak::tenant::timesheet::recent_all(&workspace_id, page, PAGE_SIZE, effective_member_id)
            .await
            .map_err(session::internal)?
    } else {
        zeitrak::tenant::timesheet::recent(&workspace_id, &user.id, page, PAGE_SIZE)
            .await
            .map_err(session::internal)?
    };

    // Build user_id → display name map when viewing all workspace timesheets.
    let member_names: HashMap<String, String> = if can_read_all {
        zeitrak::workspace::list_workspace_members(&workspace_id)
            .await
            .map_err(session::internal)?
            .into_iter()
            .map(|m| (m.user_id, m.name))
            .collect()
    } else {
        HashMap::new()
    };

    let ids: Vec<String> = rows.iter().map(|r| r.id().to_string()).collect();
    let id_refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    let mut tags_map =
        zeitrak::tenant::timesheet_tag::for_timesheets_batch(&workspace_id, &id_refs)
            .await
            .map_err(session::internal)?;

    let items = rows
        .into_iter()
        .map(|r| {
            let id = r.id().to_string();
            let member_name = if can_read_all {
                let uid = r.user_id().to_string();
                member_names.get(uid.as_str()).cloned()
            } else {
                None
            };
            let tags = tags_map
                .remove(&id)
                .unwrap_or_default()
                .into_iter()
                .map(|t| TimesheetsTagDto {
                    id: t.id().to_string(),
                    name: t.name().to_string(),
                })
                .collect();
            row_to_dto(r, tags, member_name)
        })
        .collect();

    Ok(TimesheetPageDto {
        items,
        total,
        page,
        page_size: PAGE_SIZE,
        can_filter_members: can_read_all,
    })
}

#[cfg(feature = "server")]
async fn _running_timesheet() -> Result<Option<TimesheetDto>, ServerFnError> {
    use crate::session;

    let (user, workspace_id) = session::session_workspace().await?;
    let row = zeitrak::tenant::timesheet::running(&workspace_id, &user.id)
        .await
        .map_err(session::internal)?;
    Ok(row.map(|r| row_to_dto(r, vec![], None)))
}

#[cfg(feature = "server")]
async fn _start_timesheet(
    activity_id: Option<String>,
    description: Option<String>,
) -> Result<TimesheetDto, ServerFnError> {
    use crate::session;
    use zeitrak::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::TIMESHEET_CREATE).await?;

    let r = zeitrak::tenant::timesheet::start(
        &workspace_id,
        &user.id,
        activity_id.as_deref(),
        description,
    )
    .await
    .map_err(session::internal)?;
    Ok(row_to_dto(r, vec![], None))
}

#[cfg(feature = "server")]
async fn _reassign_timesheet(
    timesheet_id: String,
    activity_id: String,
) -> Result<(), ServerFnError> {
    use crate::session;
    use zeitrak::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::TIMESHEET_UPDATE).await?;

    zeitrak::tenant::timesheet::reassign(&workspace_id, &timesheet_id, &activity_id)
        .await
        .map_err(session::internal)
}

#[cfg(feature = "server")]
async fn _update_timesheet(
    timesheet_id: String,
    description: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::session;
    use zeitrak::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::TIMESHEET_UPDATE).await?;

    zeitrak::tenant::timesheet::update(&workspace_id, &timesheet_id, description)
        .await
        .map_err(session::internal)
}

#[cfg(feature = "server")]
async fn _stop_timesheet(timesheet_id: String) -> Result<(), ServerFnError> {
    use crate::session;
    use zeitrak::core::permissions;

    // Stopping is treated as a timesheet write operation.
    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::TIMESHEET_UPDATE).await?;

    zeitrak::tenant::timesheet::stop(&workspace_id, &timesheet_id)
        .await
        .map_err(session::internal)
}

#[cfg(feature = "server")]
async fn _create_timesheet_manual(
    activity_id: Option<String>,
    start_time: String,
    end_time: String,
    description: Option<String>,
) -> Result<TimesheetDto, ServerFnError> {
    use crate::session;
    use zeitrak::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::TIMESHEET_CREATE).await?;

    let r = zeitrak::tenant::timesheet::create_manual(
        &workspace_id,
        &user.id,
        activity_id.as_deref(),
        &start_time,
        &end_time,
        description,
    )
    .await
    .map_err(session::internal)?;
    Ok(row_to_dto(r, vec![], None))
}

#[cfg(feature = "server")]
async fn _update_timesheet_time(
    timesheet_id: String,
    start_time: String,
    end_time: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::session;
    use zeitrak::core::permissions;

    let (_user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&_user, permissions::TIMESHEET_UPDATE).await?;

    zeitrak::tenant::timesheet::update_time(
        &workspace_id,
        &timesheet_id,
        &start_time,
        end_time.as_deref(),
    )
    .await
    .map_err(session::internal)
}

#[cfg(feature = "server")]
async fn _cancel_timesheet(timesheet_id: String) -> Result<(), ServerFnError> {
    use crate::session;
    use zeitrak::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::TIMESHEET_DELETE).await?;

    zeitrak::tenant::timesheet::cancel(&workspace_id, &timesheet_id)
        .await
        .map_err(session::internal)
}

#[cfg(feature = "server")]
async fn _dashboard_stats(
    member_id: Option<String>,
    period: DashboardPeriod,
) -> Result<DashboardStatsDto, ServerFnError> {
    use std::collections::HashMap;

    use chrono::{Datelike, Duration, NaiveDate, Utc};

    use crate::session;
    use zeitrak::authentication::CurrentUser;
    use zeitrak::authorization::AuthorizationService;
    use zeitrak::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    let current_user = CurrentUser {
        id: user.id.clone(),
        email: user.email.clone(),
    };
    let is_admin = AuthorizationService::is_admin(&current_user.id)
        .await
        .map_err(session::internal)?;
    let can_read_all = is_admin
        || AuthorizationService::has_permission(
            &current_user.id,
            &workspace_id,
            permissions::TIMESHEET_READ_ALL,
        )
        .await
        .map_err(session::internal)?;

    // Security: member_id filter is only applied when the caller has read_all.
    let effective_member_id: Option<&str> = if can_read_all {
        member_id.as_deref()
    } else {
        Some(user.id.as_str())
    };

    // Compute the date window based on period.
    let today = Utc::now().date_naive();
    let days_back = match period {
        DashboardPeriod::Week => 6i64,
        DashboardPeriod::Month => 27,
        DashboardPeriod::Year => 364,
    };
    let since_date = today - Duration::days(days_back);
    // Use one extra day as buffer so start-of-day timesheets are not clipped by timezone.
    let since_rfc = format!("{since_date}T00:00:00Z");

    let rows = zeitrak::tenant::timesheet::stats_for_period(
        &workspace_id,
        effective_member_id,
        &since_rfc,
    )
    .await
    .map_err(session::internal)?;

    // Build activity lookup map (id → (name, color)).
    let act_rows = zeitrak::tenant::activity::list(&workspace_id)
        .await
        .map_err(session::internal)?;
    let act_map: HashMap<String, (String, String)> = act_rows
        .iter()
        .map(|a| {
            (
                a.id().to_string(),
                (a.name().to_string(), a.color().to_string()),
            )
        })
        .collect();

    // ── KPI calculations ──────────────────────────────────────────────────────

    let days_from_monday = today.weekday().num_days_from_monday() as i64;
    let week_start = today - Duration::days(days_from_monday);

    let parse_date = |s: &str| -> Option<NaiveDate> {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.date_naive())
            .or_else(|| {
                chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f")
                    .ok()
                    .map(|dt| dt.date())
            })
    };

    let today_hours: f32 = rows
        .iter()
        .filter(|ts| ts.duration().is_some() && parse_date(ts.start_time()) == Some(today))
        .map(|ts| ts.duration().unwrap_or(0) as f32 / 3600.0)
        .sum();

    let week_hours: f32 = rows
        .iter()
        .filter(|ts| {
            ts.duration().is_some()
                && parse_date(ts.start_time())
                    .map(|d| d >= week_start)
                    .unwrap_or(false)
        })
        .map(|ts| ts.duration().unwrap_or(0) as f32 / 3600.0)
        .sum();

    // Streak: consecutive days with at least one completed timesheet, counting back from today.
    let mut streak = 0u32;
    let mut check = today;
    loop {
        let has_entry = rows
            .iter()
            .any(|ts| ts.duration().is_some() && parse_date(ts.start_time()) == Some(check));
        if has_entry {
            streak += 1;
            check -= Duration::days(1);
        } else {
            break;
        }
    }

    // ── Activity mix ──────────────────────────────────────────────────────────

    let mut mix_map: HashMap<String, f32> = HashMap::new();
    for ts in &rows {
        if let (Some(aid), Some(dur)) = (ts.activity_id().map(|id| id.to_string()), ts.duration()) {
            if dur > 0 {
                *mix_map.entry(aid).or_insert(0.0) += dur as f32 / 3600.0;
            }
        }
    }
    let total_mix_hours: f32 = mix_map.values().copied().sum();
    let mut activity_mix: Vec<ActivityMixPoint> = mix_map
        .into_iter()
        .map(|(aid, hours)| {
            let (name, color) = act_map
                .get(&aid)
                .cloned()
                .unwrap_or_else(|| ("—".to_string(), "#6c6c76".to_string()));
            let percentage = if total_mix_hours > 0.0 {
                hours / total_mix_hours * 100.0
            } else {
                0.0
            };
            ActivityMixPoint {
                activity_id: aid,
                activity_name: name,
                color,
                hours,
                percentage,
            }
        })
        .collect();
    activity_mix.sort_by(|a, b| {
        b.hours
            .partial_cmp(&a.hours)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // ── Chart bars ────────────────────────────────────────────────────────────

    let bucket_segs =
        |bucket_rows: Vec<&zeitrak::core::tenant::timesheet::TimesheetRow>| -> Vec<(String, String, String, f32)> {
            let mut map: HashMap<String, f32> = HashMap::new();
            for ts in bucket_rows {
                if let Some(dur) = ts.duration() {
                    if dur > 0 {
                        let key = ts.activity_id().map(|id| id.to_string()).unwrap_or_default();
                        *map.entry(key).or_insert(0.0) += dur as f32 / 3600.0;
                    }
                }
            }
            let mut segs: Vec<(String, String, String, f32)> = map
                .into_iter()
                .map(|(aid, hours)| {
                    let (name, color) = act_map
                        .get(&aid)
                        .cloned()
                        .unwrap_or_else(|| ("—".to_string(), "#6c6c76".to_string()));
                    (aid, name, color, hours)
                })
                .collect();
            segs.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
            segs
        };

    let chart_bars: Vec<DailyChartPoint> = match period {
        DashboardPeriod::Week => (0..7)
            .map(|i| {
                let day = today - Duration::days(6 - i as i64);
                let bucket: Vec<&zeitrak::core::tenant::timesheet::TimesheetRow> = rows
                    .iter()
                    .filter(|ts| {
                        ts.duration().is_some() && parse_date(ts.start_time()) == Some(day)
                    })
                    .collect();
                DailyChartPoint {
                    label: day.format("%a").to_string(),
                    segments: bucket_segs(bucket),
                }
            })
            .collect(),
        DashboardPeriod::Month => (0..4)
            .map(|w| {
                let week_end = today - Duration::days((3 - w as i64) * 7);
                let week_start_b = week_end - Duration::days(6);
                let bucket: Vec<&zeitrak::core::tenant::timesheet::TimesheetRow> = rows
                    .iter()
                    .filter(|ts| {
                        ts.duration().is_some()
                            && parse_date(ts.start_time())
                                .map(|d| d >= week_start_b && d <= week_end)
                                .unwrap_or(false)
                    })
                    .collect();
                DailyChartPoint {
                    label: week_start_b.format("%-d.%-m").to_string(),
                    segments: bucket_segs(bucket),
                }
            })
            .collect(),
        DashboardPeriod::Year => (0..12)
            .map(|m| {
                let month_offset = 11 - m as i32;
                let target_month =
                    ((today.month() as i32 - month_offset - 1).rem_euclid(12) + 1) as u32;
                let target_year =
                    today.year() - (month_offset + 12 - today.month() as i32).max(0) / 12;
                let bucket: Vec<&zeitrak::core::tenant::timesheet::TimesheetRow> = rows
                    .iter()
                    .filter(|ts| {
                        ts.duration().is_some()
                            && parse_date(ts.start_time())
                                .map(|d| d.year() == target_year && d.month() == target_month)
                                .unwrap_or(false)
                    })
                    .collect();
                let lbl_date =
                    NaiveDate::from_ymd_opt(today.year(), target_month, 1).unwrap_or(today);
                DailyChartPoint {
                    label: lbl_date.format("%b").to_string(),
                    segments: bucket_segs(bucket),
                }
            })
            .collect(),
    };

    Ok(DashboardStatsDto {
        today_hours,
        week_hours,
        streak,
        chart_bars,
        activity_mix,
        can_filter_members: can_read_all,
    })
}

#[cfg(feature = "server")]
async fn _monthly_overview(
    member_id: Option<String>,
) -> Result<Vec<MonthlyOverviewRow>, ServerFnError> {
    use std::collections::HashMap;

    use chrono::{Datelike, Duration, Utc};

    use crate::session;
    use zeitrak::authentication::CurrentUser;
    use zeitrak::authorization::AuthorizationService;
    use zeitrak::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    let current_user = CurrentUser {
        id: user.id.clone(),
        email: user.email.clone(),
    };
    let is_admin = AuthorizationService::is_admin(&current_user.id)
        .await
        .map_err(session::internal)?;
    let can_read_all = is_admin
        || AuthorizationService::has_permission(
            &current_user.id,
            &workspace_id,
            permissions::TIMESHEET_READ_ALL,
        )
        .await
        .map_err(session::internal)?;

    let effective_member_id: Option<&str> = if can_read_all {
        member_id.as_deref()
    } else {
        Some(user.id.as_str())
    };

    // Query the last 24 months of completed timesheets.
    let today = Utc::now().date_naive();
    let since_date = today - Duration::days(730);
    let since_rfc = format!("{since_date}T00:00:00Z");

    let rows = zeitrak::tenant::timesheet::stats_for_period(
        &workspace_id,
        effective_member_id,
        &since_rfc,
    )
    .await
    .map_err(session::internal)?;

    let act_rows = zeitrak::tenant::activity::list(&workspace_id)
        .await
        .map_err(session::internal)?;
    let act_map: HashMap<String, (String, String)> = act_rows
        .iter()
        .map(|a| {
            (
                a.id().to_string(),
                (a.name().to_string(), a.color().to_string()),
            )
        })
        .collect();

    let parse_ym = |s: &str| -> Option<(i32, u32)> {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| (dt.year(), dt.month()))
            .or_else(|| {
                chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f")
                    .ok()
                    .map(|dt| (dt.year(), dt.month()))
            })
    };

    // Accumulate hours per (year, month, activity_id).
    let mut monthly: HashMap<(i32, u32), HashMap<String, f32>> = HashMap::new();
    for ts in &rows {
        if let (Some(ym), Some(dur)) = (parse_ym(ts.start_time()), ts.duration()) {
            if dur > 0 {
                let aid = ts.activity_id().map(|id| id.to_string()).unwrap_or_default();
                *monthly.entry(ym).or_default().entry(aid).or_insert(0.0) +=
                    dur as f32 / 3600.0;
            }
        }
    }

    let mut result: Vec<MonthlyOverviewRow> = monthly
        .into_iter()
        .map(|((year, month), mix_map)| {
            let total_hours: f32 = mix_map.values().copied().sum();
            let mut activity_mix: Vec<ActivityMixPoint> = mix_map
                .into_iter()
                .map(|(aid, hours)| {
                    let (name, color) = act_map
                        .get(&aid)
                        .cloned()
                        .unwrap_or_else(|| ("—".to_string(), "#6c6c76".to_string()));
                    ActivityMixPoint {
                        activity_id: aid,
                        activity_name: name,
                        color,
                        hours,
                        percentage: if total_hours > 0.0 {
                            hours / total_hours * 100.0
                        } else {
                            0.0
                        },
                    }
                })
                .collect();
            activity_mix.sort_by(|a, b| {
                b.hours
                    .partial_cmp(&a.hours)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            MonthlyOverviewRow {
                year,
                month,
                total_hours,
                activity_mix,
            }
        })
        .collect();

    result.sort_by(|a, b| b.year.cmp(&a.year).then(b.month.cmp(&a.month)));
    Ok(result)
}

#[cfg(feature = "server")]
async fn _get_timeline_entries(
    page: u32,
    from: Option<String>,
    to: Option<String>,
    member_id: Option<String>,
) -> Result<TimesheetPageDto, ServerFnError> {
    use std::collections::HashMap;

    use crate::session;
    use zeitrak::authentication::CurrentUser;
    use zeitrak::authorization::AuthorizationService;
    use zeitrak::core::permissions;

    const PAGE_SIZE: u32 = 50;

    let (user, workspace_id) = session::session_workspace().await?;
    let current_user = CurrentUser {
        id: user.id.clone(),
        email: user.email.clone(),
    };
    let is_admin = AuthorizationService::is_admin(&current_user.id)
        .await
        .map_err(session::internal)?;
    let can_read_all = is_admin
        || AuthorizationService::has_permission(
            &current_user.id,
            &workspace_id,
            permissions::TIMESHEET_READ_ALL,
        )
        .await
        .map_err(session::internal)?;

    let effective_member_id: Option<&str> =
        if can_read_all { member_id.as_deref() } else { Some(&user.id) };

    let (rows, total) = zeitrak::tenant::timesheet::timeline_entries(
        &workspace_id,
        page,
        PAGE_SIZE,
        from.as_deref(),
        to.as_deref(),
        effective_member_id,
    )
    .await
    .map_err(session::internal)?;

    let member_names: HashMap<String, String> = if can_read_all {
        zeitrak::workspace::list_workspace_members(&workspace_id)
            .await
            .map_err(session::internal)?
            .into_iter()
            .map(|m| (m.user_id, m.name))
            .collect()
    } else {
        HashMap::new()
    };

    let ids: Vec<String> = rows.iter().map(|r| r.id().to_string()).collect();
    let id_refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    let mut tags_map =
        zeitrak::tenant::timesheet_tag::for_timesheets_batch(&workspace_id, &id_refs)
            .await
            .map_err(session::internal)?;

    let items = rows
        .into_iter()
        .map(|r| {
            let id = r.id().to_string();
            let member_name = if can_read_all {
                let uid = r.user_id().to_string();
                member_names.get(uid.as_str()).cloned()
            } else {
                None
            };
            let tags = tags_map
                .remove(&id)
                .unwrap_or_default()
                .into_iter()
                .map(|t| TimesheetsTagDto {
                    id: t.id().to_string(),
                    name: t.name().to_string(),
                })
                .collect();
            row_to_dto(r, tags, member_name)
        })
        .collect();

    Ok(TimesheetPageDto {
        items,
        total,
        page,
        page_size: PAGE_SIZE,
        can_filter_members: can_read_all,
    })
}

#[cfg(feature = "server")]
async fn _get_timeline_stats(
    from: Option<String>,
    to: Option<String>,
    member_id: Option<String>,
) -> Result<TimelineStatsDto, ServerFnError> {
    use std::collections::HashMap;

    use crate::session;
    use zeitrak::authentication::CurrentUser;
    use zeitrak::authorization::AuthorizationService;
    use zeitrak::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    let current_user = CurrentUser {
        id: user.id.clone(),
        email: user.email.clone(),
    };
    let is_admin = AuthorizationService::is_admin(&current_user.id)
        .await
        .map_err(session::internal)?;
    let can_read_all = is_admin
        || AuthorizationService::has_permission(
            &current_user.id,
            &workspace_id,
            permissions::TIMESHEET_READ_ALL,
        )
        .await
        .map_err(session::internal)?;

    let effective_member_id: Option<&str> =
        if can_read_all { member_id.as_deref() } else { Some(&user.id) };

    let rows = zeitrak::tenant::timesheet::timeline_stats(
        &workspace_id,
        from.as_deref(),
        to.as_deref(),
        effective_member_id,
    )
    .await
    .map_err(session::internal)?;

    let act_rows = zeitrak::tenant::activity::list(&workspace_id)
        .await
        .map_err(session::internal)?;
    let act_map: HashMap<String, (String, String)> = act_rows
        .iter()
        .map(|a| {
            (
                a.id().to_string(),
                (a.name().to_string(), a.color().to_string()),
            )
        })
        .collect();

    let mut by_id: HashMap<String, i64> = HashMap::new();
    let mut total_seconds: i64 = 0;
    for r in &rows {
        if let Some(dur) = r.duration() {
            if dur > 0 {
                let aid = r
                    .activity_id()
                    .map(|id| id.to_string())
                    .unwrap_or_default();
                *by_id.entry(aid).or_insert(0) += i64::from(dur);
                total_seconds += i64::from(dur);
            }
        }
    }

    let mut by_activity: Vec<ActivityStatDto> = by_id
        .into_iter()
        .map(|(aid, secs)| {
            let (name, color) = act_map
                .get(&aid)
                .cloned()
                .unwrap_or_else(|| ("—".to_string(), "#6c6c76".to_string()));
            let percentage = if total_seconds > 0 {
                secs as f32 / total_seconds as f32 * 100.0
            } else {
                0.0
            };
            ActivityStatDto {
                activity_id: aid,
                activity_name: name,
                color,
                total_seconds: secs,
                percentage,
            }
        })
        .collect();
    by_activity.sort_by(|a, b| b.total_seconds.cmp(&a.total_seconds));

    Ok(TimelineStatsDto {
        total_seconds,
        by_activity,
        can_filter_members: can_read_all,
    })
}
