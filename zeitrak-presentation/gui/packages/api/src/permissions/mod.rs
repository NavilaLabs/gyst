use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionDto {
    pub id: String,
    pub name: String,
}

/// Returns all permissions available in the system.
#[get("/api/permissions")]
pub async fn list_permissions() -> Result<Vec<PermissionDto>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _list_permissions().await
    }
    #[cfg(not(feature = "server"))]
    {
        Ok(vec![])
    }
}

#[cfg(feature = "server")]
async fn _list_permissions() -> Result<Vec<PermissionDto>, ServerFnError> {
    use crate::session::{internal, session_workspace};

    let _ = session_workspace().await?;
    let perms = zeitrak::workspace::list_all_permissions()
        .await
        .map_err(internal)?;

    Ok(perms
        .into_iter()
        .map(|p| PermissionDto {
            id: p.id().to_string(),
            name: p.name().to_string(),
        })
        .collect())
}
