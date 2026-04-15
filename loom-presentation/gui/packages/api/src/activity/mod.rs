use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityDto {
    pub id: String,
    pub name: String,
    pub comment: Option<String>,
}

#[get("/api/activities")]
pub async fn list_activities() -> Result<Vec<ActivityDto>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _list_activities().await
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::ServerError {
            message: "server only".into(),
            code: 500,
            details: None,
        })
    }
}

#[post("/api/activities")]
pub async fn create_activity(name: String) -> Result<ActivityDto, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _create_activity(name).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (name);
        Err(ServerFnError::ServerError {
            message: "server only".into(),
            code: 500,
            details: None,
        })
    }
}

#[post("/api/activities/update")]
pub async fn update_activity(
    id: String,
    name: String,
    comment: Option<String>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _update_activity(id, name, comment).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (id, name, comment);
        Err(ServerFnError::ServerError {
            message: "server only".into(),
            code: 500,
            details: None,
        })
    }
}

#[cfg(feature = "server")]
async fn _list_activities() -> Result<Vec<ActivityDto>, ServerFnError> {
    use crate::session;

    let (_, workspace_id) = session::session_workspace().await?;
    let rows = loom::tenant::activity::list(&workspace_id)
        .await
        .map_err(session::internal)?;
    Ok(rows
        .into_iter()
        .map(|r| ActivityDto {
            id: r.id().to_string(),
            name: r.name().to_string(),
            comment: r.comment().map(String::from),
        })
        .collect())
}

#[cfg(feature = "server")]
async fn _create_activity(name: String) -> Result<ActivityDto, ServerFnError> {
    use crate::session;
    use loom::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::ACTIVITY_CREATE).await?;

    let r = loom::tenant::activity::create(&workspace_id, name, None)
        .await
        .map_err(session::internal)?;
    Ok(ActivityDto {
        id: r.id().to_string(),
        name: r.name().to_string(),
        comment: r.comment().map(String::from),
    })
}

#[cfg(feature = "server")]
async fn _update_activity(
    id: String,
    name: String,
    comment: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::session;
    use loom::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::ACTIVITY_UPDATE).await?;

    loom::tenant::activity::update(&workspace_id, &id, name, comment)
        .await
        .map_err(session::internal)
}
