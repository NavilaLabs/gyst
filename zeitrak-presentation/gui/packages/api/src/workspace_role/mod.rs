use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceRoleDto {
    pub id: String,
    pub name: String,
}

/// Returns all workspace roles for the current workspace.
#[get("/api/workspace-roles")]
pub async fn list_workspace_roles() -> Result<Vec<WorkspaceRoleDto>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _list_workspace_roles().await
    }
    #[cfg(not(feature = "server"))]
    {
        Ok(vec![])
    }
}

#[cfg(feature = "server")]
async fn _list_workspace_roles() -> Result<Vec<WorkspaceRoleDto>, ServerFnError> {
    use crate::session::{internal, session_workspace};

    let (_, workspace_id) = session_workspace().await?;
    let roles = zeitrak::workspace::list_workspace_roles(&workspace_id)
        .await
        .map_err(internal)?;

    Ok(roles
        .into_iter()
        .map(|r| WorkspaceRoleDto {
            id: r.id().to_string(),
            name: r.name().unwrap_or("Unnamed role").to_owned(),
        })
        .collect())
}
