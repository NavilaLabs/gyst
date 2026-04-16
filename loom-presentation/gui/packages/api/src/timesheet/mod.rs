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
}

#[get("/api/timesheets/recent")]
pub async fn list_timesheets() -> Result<Vec<TimesheetDto>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _list_timesheets().await
    }
    #[cfg(not(feature = "server"))]
    {
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

#[cfg(feature = "server")]
fn row_to_dto(
    r: loom::core::tenant::timesheet::TimesheetRow,
    tags: Vec<TimesheetsTagDto>,
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
    }
}

#[cfg(feature = "server")]
async fn _list_timesheets() -> Result<Vec<TimesheetDto>, ServerFnError> {
    use crate::session;

    let (user, workspace_id) = session::session_workspace().await?;
    let rows = loom::tenant::timesheet::recent(&workspace_id, &user.id)
        .await
        .map_err(session::internal)?;

    let ids: Vec<String> = rows.iter().map(|r| r.id().to_string()).collect();
    let id_refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    let mut tags_map = loom::tenant::timesheet_tag::for_timesheets_batch(&workspace_id, &id_refs)
        .await
        .map_err(session::internal)?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let id = r.id().to_string();
            let tags = tags_map
                .remove(&id)
                .unwrap_or_default()
                .into_iter()
                .map(|t| TimesheetsTagDto {
                    id: t.id().to_string(),
                    name: t.name().to_string(),
                })
                .collect();
            row_to_dto(r, tags)
        })
        .collect())
}

#[cfg(feature = "server")]
async fn _running_timesheet() -> Result<Option<TimesheetDto>, ServerFnError> {
    use crate::session;

    let (user, workspace_id) = session::session_workspace().await?;
    let row = loom::tenant::timesheet::running(&workspace_id, &user.id)
        .await
        .map_err(session::internal)?;
    Ok(row.map(|r| row_to_dto(r, vec![])))
}

#[cfg(feature = "server")]
async fn _start_timesheet(
    activity_id: Option<String>,
    description: Option<String>,
) -> Result<TimesheetDto, ServerFnError> {
    use crate::session;
    use loom::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::TIMESHEET_CREATE).await?;

    let r = loom::tenant::timesheet::start(
        &workspace_id,
        &user.id,
        activity_id.as_deref(),
        description,
    )
    .await
    .map_err(session::internal)?;
    Ok(row_to_dto(r, vec![]))
}

#[cfg(feature = "server")]
async fn _reassign_timesheet(
    timesheet_id: String,
    activity_id: String,
) -> Result<(), ServerFnError> {
    use crate::session;
    use loom::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::TIMESHEET_UPDATE).await?;

    loom::tenant::timesheet::reassign(&workspace_id, &timesheet_id, &activity_id)
        .await
        .map_err(session::internal)
}

#[cfg(feature = "server")]
async fn _update_timesheet(
    timesheet_id: String,
    description: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::session;
    use loom::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::TIMESHEET_UPDATE).await?;

    loom::tenant::timesheet::update(&workspace_id, &timesheet_id, description)
        .await
        .map_err(session::internal)
}

#[cfg(feature = "server")]
async fn _stop_timesheet(timesheet_id: String) -> Result<(), ServerFnError> {
    use crate::session;
    use loom::core::permissions;

    // Stopping is treated as a timesheet write operation.
    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::TIMESHEET_UPDATE).await?;

    loom::tenant::timesheet::stop(&workspace_id, &timesheet_id)
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
    use loom::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::TIMESHEET_CREATE).await?;

    let r = loom::tenant::timesheet::create_manual(
        &workspace_id,
        &user.id,
        activity_id.as_deref(),
        &start_time,
        &end_time,
        description,
    )
    .await
    .map_err(session::internal)?;
    Ok(row_to_dto(r, vec![]))
}

#[cfg(feature = "server")]
async fn _update_timesheet_time(
    timesheet_id: String,
    start_time: String,
    end_time: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::session;
    use loom::core::permissions;

    let (_user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&_user, permissions::TIMESHEET_UPDATE).await?;

    loom::tenant::timesheet::update_time(
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
    use loom::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::TIMESHEET_CANCEL).await?;

    loom::tenant::timesheet::cancel(&workspace_id, &timesheet_id)
        .await
        .map_err(session::internal)
}
