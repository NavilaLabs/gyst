use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceRoleDto {
    pub id: String,
    pub name: String,
    /// Permission names assigned to this role (populated by `list_roles_with_permissions`).
    pub permissions: Vec<String>,
}

/// Returns all workspace roles for the current workspace (without permission details).
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
    use crate::session::{internal, require_permission, session_workspace};
    use zeitrak::core::permissions::ROLE_READ;

    let (user, workspace_id) = session_workspace().await?;
    require_permission(&user, ROLE_READ).await?;

    let roles = zeitrak::workspace::list_workspace_roles(&workspace_id)
        .await
        .map_err(internal)?;

    Ok(roles
        .into_iter()
        .map(|r| WorkspaceRoleDto {
            id: r.id().to_string(),
            name: r.name().unwrap_or("Unnamed role").to_owned(),
            permissions: vec![],
        })
        .collect())
}

/// Returns all workspace roles enriched with their permission names.
#[get("/api/workspace-roles/with-permissions")]
pub async fn list_roles_with_permissions() -> Result<Vec<WorkspaceRoleDto>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _list_roles_with_permissions().await
    }
    #[cfg(not(feature = "server"))]
    {
        Ok(vec![])
    }
}

#[cfg(feature = "server")]
async fn _list_roles_with_permissions() -> Result<Vec<WorkspaceRoleDto>, ServerFnError> {
    use crate::session::{internal, require_permission, session_workspace};
    use zeitrak::core::permissions::ROLE_READ;

    let (user, workspace_id) = session_workspace().await?;
    require_permission(&user, ROLE_READ).await?;

    let roles = zeitrak::workspace::list_roles_with_permissions(&workspace_id)
        .await
        .map_err(internal)?;

    Ok(roles
        .into_iter()
        .map(|r| WorkspaceRoleDto {
            id: r.id,
            name: r.name.unwrap_or_else(|| "Unnamed role".to_string()),
            permissions: r.permission_names,
        })
        .collect())
}

/// Creates a new role in the current workspace.
///
/// Requires the `role.create` permission.
#[post("/api/workspace-roles/create")]
pub async fn create_role(name: String) -> Result<String, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _create_role(name).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = name;
        Ok(String::new())
    }
}

#[cfg(feature = "server")]
async fn _create_role(name: String) -> Result<String, ServerFnError> {
    use crate::session::{internal, require_permission, session_workspace};
    use zeitrak::core::permissions::ROLE_CREATE;

    let (user, workspace_id) = session_workspace().await?;
    require_permission(&user, ROLE_CREATE).await?;

    let role_id = zeitrak::workspace::create_role(&workspace_id, name)
        .await
        .map_err(internal)?;

    Ok(role_id.to_string())
}

/// Renames an existing workspace role.
///
/// Requires the `role.update` permission.
#[post("/api/workspace-roles/rename")]
pub async fn rename_role(role_id: String, name: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _rename_role(role_id, name).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (role_id, name);
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn _rename_role(role_id: String, name: String) -> Result<(), ServerFnError> {
    use crate::session::{internal, require_permission, session_workspace};
    use zeitrak::core::permissions::ROLE_UPDATE;

    let (user, _) = session_workspace().await?;
    require_permission(&user, ROLE_UPDATE).await?;

    zeitrak::workspace::rename_role(&role_id, name)
        .await
        .map_err(internal)
}

/// Deletes a workspace role.
///
/// Returns an error if any members still have this role assigned.
/// Requires the `role.delete` permission.
#[post("/api/workspace-roles/delete")]
pub async fn delete_role(role_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _delete_role(role_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = role_id;
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn _delete_role(role_id: String) -> Result<(), ServerFnError> {
    use crate::session::{internal, require_permission, session_workspace};
    use zeitrak::core::permissions::ROLE_DELETE;

    let (user, _) = session_workspace().await?;
    require_permission(&user, ROLE_DELETE).await?;

    zeitrak::workspace::delete_role(&role_id)
        .await
        .map_err(internal)
}

/// Grants a permission to a workspace role.
///
/// Requires the `role.update` permission.
#[post("/api/workspace-roles/grant-permission")]
pub async fn grant_role_permission(
    role_id: String,
    permission_id: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _grant_role_permission(role_id, permission_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (role_id, permission_id);
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn _grant_role_permission(
    role_id: String,
    permission_id: String,
) -> Result<(), ServerFnError> {
    use crate::session::{internal, require_permission, session_workspace};
    use zeitrak::core::permissions::ROLE_UPDATE;

    let (user, _) = session_workspace().await?;
    require_permission(&user, ROLE_UPDATE).await?;

    zeitrak::workspace::grant_role_permission(&role_id, &permission_id)
        .await
        .map_err(internal)
}

/// Revokes a permission from a workspace role.
///
/// Requires the `role.update` permission.
#[post("/api/workspace-roles/revoke-permission")]
pub async fn revoke_role_permission(
    role_id: String,
    permission_id: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _revoke_role_permission(role_id, permission_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (role_id, permission_id);
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn _revoke_role_permission(
    role_id: String,
    permission_id: String,
) -> Result<(), ServerFnError> {
    use crate::session::{internal, require_permission, session_workspace};
    use zeitrak::core::permissions::ROLE_UPDATE;

    let (user, _) = session_workspace().await?;
    require_permission(&user, ROLE_UPDATE).await?;

    zeitrak::workspace::revoke_role_permission(&role_id, &permission_id)
        .await
        .map_err(internal)
}
