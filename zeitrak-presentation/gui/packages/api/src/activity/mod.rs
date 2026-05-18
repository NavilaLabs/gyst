use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityDto {
    pub id: String,
    pub name: String,
    pub color: String,
    pub comment: Option<String>,
}

#[server]
#[get("/api/activities")]
pub async fn list_activities() -> Result<Vec<ActivityDto>, ServerFnError> {
    use crate::session;

    let (_, workspace_id) = session::session_workspace().await?;
    let rows = zeitrak::tenant::activity::list(&workspace_id)
        .await
        .map_err(session::internal)?;
    Ok(rows
        .into_iter()
        .map(|r| ActivityDto {
            id: r.id().to_string(),
            name: r.name().to_string(),
            color: r.color().to_string(),
            comment: r.comment().map(String::from),
        })
        .collect())
}

#[server]
#[post("/api/activities")]
pub async fn create_activity(name: String, color: String) -> Result<ActivityDto, ServerFnError> {
    use crate::session;
    use zeitrak::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::ACTIVITY_CREATE).await?;

    let r = zeitrak::tenant::activity::create(&workspace_id, name, color, None)
        .await
        .map_err(session::internal)?;
    Ok(ActivityDto {
        id: r.id().to_string(),
        name: r.name().to_string(),
        color: r.color().to_string(),
        comment: r.comment().map(String::from),
    })
}

#[server]
#[post("/api/activities/delete")]
pub async fn delete_activity(id: String) -> Result<(), ServerFnError> {
    use crate::session;
    use zeitrak::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::ACTIVITY_DELETE).await?;

    zeitrak::tenant::activity::delete(&workspace_id, &id)
        .await
        .map_err(session::internal)
}

#[server]
#[post("/api/activities/update")]
pub async fn update_activity(
    id: String,
    name: String,
    color: String,
    comment: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::session;
    use zeitrak::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::ACTIVITY_UPDATE).await?;

    zeitrak::tenant::activity::update(&workspace_id, &id, name, color, comment)
        .await
        .map_err(session::internal)
}
