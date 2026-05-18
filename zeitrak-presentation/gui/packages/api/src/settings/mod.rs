use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserSettingsDto {
    pub timezone: String,
    pub date_format: String,
    pub language: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceSettingsDto {
    pub name: Option<String>,
    pub timezone: String,
    pub date_format: String,
    pub currency: String,
    pub week_start: String,
}

/// Returns the settings of the currently authenticated user.
#[server]
#[get("/api/settings/user")]
pub async fn get_user_settings() -> Result<UserSettingsDto, ServerFnError> {
    use crate::session;

    let user = session::session_user().await?;
    let view = zeitrak::user_settings::get_user_settings(&user.id)
        .await
        .map_err(session::internal)?;
    Ok(UserSettingsDto {
        timezone: view.timezone,
        date_format: view.date_format,
        language: view.language,
    })
}

/// Saves settings for the currently authenticated user.
#[server]
#[post("/api/settings/user")]
pub async fn update_user_settings(
    timezone: String,
    date_format: String,
    language: String,
) -> Result<(), ServerFnError> {
    use crate::session;

    let user = session::session_user().await?;
    zeitrak::user_settings::update_user_settings(&user.id, timezone, date_format, language)
        .await
        .map_err(session::internal)
}

/// Returns the settings of the currently selected workspace.
#[server]
#[get("/api/settings/workspace")]
pub async fn get_workspace_settings() -> Result<WorkspaceSettingsDto, ServerFnError> {
    use crate::session;

    let (_user, workspace_id) = session::session_workspace().await?;
    let view = zeitrak::workspace::get_workspace_settings(&workspace_id)
        .await
        .map_err(session::internal)?;
    Ok(WorkspaceSettingsDto {
        name: view.name().map(ToString::to_string),
        timezone: view.timezone,
        date_format: view.date_format,
        currency: view.currency,
        week_start: view.week_start,
    })
}

/// Saves settings for the currently selected workspace.
#[server]
#[post("/api/settings/workspace")]
pub async fn update_workspace_settings(
    name: Option<String>,
    timezone: String,
    date_format: String,
    currency: String,
    week_start: String,
) -> Result<(), ServerFnError> {
    use crate::session;

    let (_user, workspace_id) = session::session_workspace().await?;
    zeitrak::workspace::update_workspace_settings(
        &workspace_id,
        name,
        timezone,
        date_format,
        currency,
        week_start,
    )
    .await
    .map_err(session::internal)
}
