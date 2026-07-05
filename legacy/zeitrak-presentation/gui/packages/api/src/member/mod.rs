use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberDto {
    pub user_id: String,
    pub name: String,
    pub email: String,
    /// IDs of roles assigned to this member in the current workspace.
    pub role_ids: Vec<String>,
    /// IDs of permissions directly granted to this member in the current workspace.
    pub permission_ids: Vec<String>,
}

/// Returns all members of the current workspace.
#[get("/api/members")]
pub async fn list_members() -> Result<Vec<MemberDto>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _list_members().await
    }
    #[cfg(not(feature = "server"))]
    {
        Ok(vec![])
    }
}

#[cfg(feature = "server")]
async fn _list_members() -> Result<Vec<MemberDto>, ServerFnError> {
    use crate::session::{internal, require_permission, session_workspace};
    use zeitrak::core::permissions::MEMBER_READ;

    let (user, workspace_id) = session_workspace().await?;
    require_permission(&user, MEMBER_READ).await?;

    let members = zeitrak::workspace::list_workspace_members(&workspace_id)
        .await
        .map_err(internal)?;

    Ok(members
        .into_iter()
        .map(|m| MemberDto {
            user_id: m.user_id,
            name: m.name,
            email: m.email,
            role_ids: m.role_ids,
            permission_ids: m.permission_ids,
        })
        .collect())
}

/// Assigns a role to a workspace member.
///
/// Requires the `member.update` permission.
#[post("/api/members/assign-role")]
pub async fn assign_member_role(user_id: String, role_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _assign_member_role(user_id, role_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (user_id, role_id);
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn _assign_member_role(user_id: String, role_id: String) -> Result<(), ServerFnError> {
    use crate::session::{internal, require_permission, session_workspace};
    use zeitrak::core::permissions::MEMBER_UPDATE;

    let (user, workspace_id) = session_workspace().await?;
    require_permission(&user, MEMBER_UPDATE).await?;

    zeitrak::workspace::assign_member_role(&workspace_id, &user_id, &role_id)
        .await
        .map_err(internal)
}

/// Revokes a role from a workspace member.
///
/// Requires the `member.update` permission.
#[post("/api/members/revoke-role")]
pub async fn revoke_member_role(user_id: String, role_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _revoke_member_role(user_id, role_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (user_id, role_id);
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn _revoke_member_role(user_id: String, role_id: String) -> Result<(), ServerFnError> {
    use crate::session::{internal, require_permission, session_workspace};
    use zeitrak::core::permissions::MEMBER_UPDATE;

    let (user, workspace_id) = session_workspace().await?;
    require_permission(&user, MEMBER_UPDATE).await?;

    zeitrak::workspace::revoke_member_role(&workspace_id, &user_id, &role_id)
        .await
        .map_err(internal)
}

/// Grants a direct permission to a workspace member.
///
/// Requires the `member.update` permission.
#[post("/api/members/grant-permission")]
pub async fn grant_member_permission(
    user_id: String,
    permission_id: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _grant_member_permission(user_id, permission_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (user_id, permission_id);
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn _grant_member_permission(
    user_id: String,
    permission_id: String,
) -> Result<(), ServerFnError> {
    use crate::session::{internal, require_permission, session_workspace};
    use zeitrak::core::permissions::MEMBER_UPDATE;

    let (user, workspace_id) = session_workspace().await?;
    require_permission(&user, MEMBER_UPDATE).await?;

    zeitrak::workspace::grant_member_permission(&workspace_id, &user_id, &permission_id)
        .await
        .map_err(internal)
}

/// Revokes a direct permission from a workspace member.
///
/// Requires the `member.update` permission.
#[post("/api/members/revoke-permission")]
pub async fn revoke_member_permission(
    user_id: String,
    permission_id: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _revoke_member_permission(user_id, permission_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (user_id, permission_id);
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn _revoke_member_permission(
    user_id: String,
    permission_id: String,
) -> Result<(), ServerFnError> {
    use crate::session::{internal, require_permission, session_workspace};
    use zeitrak::core::permissions::MEMBER_UPDATE;

    let (user, workspace_id) = session_workspace().await?;
    require_permission(&user, MEMBER_UPDATE).await?;

    zeitrak::workspace::revoke_member_permission(&workspace_id, &user_id, &permission_id)
        .await
        .map_err(internal)
}

/// Removes a member from the current workspace.
///
/// All their role and direct permission assignments in this workspace are revoked.
/// Returns an error if the user is the last admin.
/// Requires the `member.delete` permission.
#[post("/api/members/remove")]
pub async fn remove_member(user_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _remove_member(user_id).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = user_id;
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn _remove_member(user_id: String) -> Result<(), ServerFnError> {
    use crate::session::{internal, require_permission, session_workspace};
    use zeitrak::core::permissions::MEMBER_DELETE;

    let (user, workspace_id) = session_workspace().await?;
    require_permission(&user, MEMBER_DELETE).await?;

    zeitrak::workspace::remove_member(&workspace_id, &user_id)
        .await
        .map_err(internal)
}
