use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimesheetsTagDto {
    pub id: String,
    pub name: String,
}

#[get("/api/tags")]
pub async fn list_tags() -> Result<Vec<TimesheetsTagDto>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _list_tags().await
    }
    #[cfg(not(feature = "server"))]
    {
        Ok(vec![])
    }
}

#[post("/api/tags/for-timesheet")]
pub async fn list_timesheet_tags(
    timesheet_id: String,
) -> Result<Vec<TimesheetsTagDto>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _list_timesheet_tags(timesheet_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = timesheet_id;
        Ok(vec![])
    }
}

#[post("/api/tags")]
pub async fn create_tag(name: String) -> Result<TimesheetsTagDto, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _create_tag(name).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = name;
        Err(ServerFnError::ServerError {
            message: "server only".into(),
            code: 500,
            details: None,
        })
    }
}

#[post("/api/tags/rename")]
pub async fn rename_tag(id: String, name: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _rename_tag(id, name).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (id, name);
        Ok(())
    }
}

#[post("/api/tags/tag-timesheet")]
pub async fn tag_timesheet(tag_id: String, timesheet_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _tag_timesheet(tag_id, timesheet_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (tag_id, timesheet_id);
        Ok(())
    }
}

#[post("/api/tags/untag-timesheet")]
pub async fn untag_timesheet(tag_id: String, timesheet_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _untag_timesheet(tag_id, timesheet_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (tag_id, timesheet_id);
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn _list_tags() -> Result<Vec<TimesheetsTagDto>, ServerFnError> {
    use crate::session;

    let (_, workspace_id) = session::session_workspace().await?;
    loom::tenant::timesheet_tag::list(&workspace_id)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|r| TimesheetsTagDto {
                    id: r.id().to_string(),
                    name: r.name().to_string(),
                })
                .collect()
        })
        .map_err(session::internal)
}

#[cfg(feature = "server")]
async fn _list_timesheet_tags(
    timesheet_id: String,
) -> Result<Vec<TimesheetsTagDto>, ServerFnError> {
    use crate::session;

    let (_, workspace_id) = session::session_workspace().await?;
    loom::tenant::timesheet_tag::list_for_timesheet(&workspace_id, &timesheet_id)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|r| TimesheetsTagDto {
                    id: r.id().to_string(),
                    name: r.name().to_string(),
                })
                .collect()
        })
        .map_err(session::internal)
}

#[cfg(feature = "server")]
async fn _create_tag(name: String) -> Result<TimesheetsTagDto, ServerFnError> {
    use crate::session;
    use loom::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::TAG_MANAGE).await?;

    loom::tenant::timesheet_tag::create(&workspace_id, name)
        .await
        .map(|r| TimesheetsTagDto {
            id: r.id().to_string(),
            name: r.name().to_string(),
        })
        .map_err(session::internal)
}

#[cfg(feature = "server")]
async fn _rename_tag(id: String, name: String) -> Result<(), ServerFnError> {
    use crate::session;
    use loom::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::TAG_MANAGE).await?;

    loom::tenant::timesheet_tag::rename(&workspace_id, &id, name)
        .await
        .map_err(session::internal)
}

#[cfg(feature = "server")]
async fn _tag_timesheet(tag_id: String, timesheet_id: String) -> Result<(), ServerFnError> {
    use crate::session;
    use loom::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::TAG_MANAGE).await?;

    loom::tenant::timesheet_tag::tag_timesheet(&workspace_id, &tag_id, &timesheet_id)
        .await
        .map_err(session::internal)
}

#[cfg(feature = "server")]
async fn _untag_timesheet(tag_id: String, timesheet_id: String) -> Result<(), ServerFnError> {
    use crate::session;
    use loom::core::permissions;

    let (user, workspace_id) = session::session_workspace().await?;
    session::require_permission(&user, permissions::TAG_MANAGE).await?;

    loom::tenant::timesheet_tag::untag_timesheet(&workspace_id, &tag_id, &timesheet_id)
        .await
        .map_err(session::internal)
}
