use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceDto {
    pub id: String,
    pub name: Option<String>,
}

/// Returns the workspaces available to the currently authenticated user.
#[get("/api/workspaces")]
pub async fn list_workspaces() -> Result<Vec<WorkspaceDto>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _list_workspaces().await
    }
    #[cfg(not(feature = "server"))]
    {
        Ok(vec![])
    }
}

/// Stores the selected workspace_id in the session.
#[post("/api/workspaces/select")]
pub async fn select_workspace(workspace_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _select_workspace(workspace_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = workspace_id;
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn _list_workspaces() -> Result<Vec<WorkspaceDto>, ServerFnError> {
    use crate::auth::UserInfo;
    use dioxus::fullstack::extract;
    use tower_sessions::Session;

    let session: Session = extract().await?;
    let user: Option<UserInfo> =
        session
            .get("user")
            .await
            .map_err(|e| ServerFnError::ServerError {
                message: e.to_string(),
                code: 500,
                details: None,
            })?;
    let user = user.ok_or_else(|| ServerFnError::ServerError {
        message: "not authenticated".into(),
        code: 401,
        details: None,
    })?;

    let workspaces = zeitrak::workspace::list_user_workspaces(&user.id)
        .await
        .map_err(|e| ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        })?;

    Ok(workspaces
        .into_iter()
        .map(|w| WorkspaceDto {
            id: w.id,
            name: w.name,
        })
        .collect())
}

#[cfg(feature = "server")]
async fn _select_workspace(workspace_id: String) -> Result<(), ServerFnError> {
    use crate::auth::UserInfo;
    use dioxus::fullstack::extract;
    use tower_sessions::Session;

    let session: Session = extract().await?;
    let user: Option<UserInfo> =
        session
            .get("user")
            .await
            .map_err(|e| ServerFnError::ServerError {
                message: e.to_string(),
                code: 500,
                details: None,
            })?;
    let mut user = user.ok_or_else(|| ServerFnError::ServerError {
        message: "not authenticated".into(),
        code: 401,
        details: None,
    })?;

    // Verify the workspace belongs to this user.
    let available = zeitrak::workspace::list_user_workspaces(&user.id)
        .await
        .map_err(|e| ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        })?;
    if !available.iter().any(|w| w.id == workspace_id) {
        return Err(ServerFnError::ServerError {
            message: "workspace not found for this user".into(),
            code: 403,
            details: None,
        });
    }

    user.workspace_id = Some(workspace_id);
    session
        .insert("user", user)
        .await
        .map_err(|e| ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        })
}
