use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvitationDto {
    pub id: String,
    pub workspace_id: String,
    pub workspace_name: Option<String>,
    pub email: String,
    pub workspace_role_id: String,
    pub token: String,
    pub status: String,
}

/// Sends a workspace invitation to an email address.
///
/// Requires the `member.invite` permission in the current workspace.
/// Returns the new invitation ID on success.
#[server]
#[post("/api/invitations/send")]
pub async fn send_invitation(
    email: String,
    workspace_role_id: String,
    ttl_days: u32,
) -> Result<String, ServerFnError> {
    use crate::session::{internal, require_permission, session_workspace};

    let (user, workspace_id) = session_workspace().await?;
    require_permission(&user, zeitrak::core::permissions::MEMBER_INVITE).await?;

    let role_id: zeitrak::core::admin::workspace_role::WorkspaceRoleId = workspace_role_id
        .parse()
        .map_err(|_| ServerFnError::ServerError {
            message: "invalid workspace_role_id".into(),
            code: 400,
            details: None,
        })?;

    let email_sender = zeitrak::email::email_sender_from_config().await.map_err(internal)?;
    let base_url = zeitrak::email::base_url();

    let current_user = zeitrak::auth::CurrentUser {
        id: user.id,
        email: user.email,
    };

    let invitation_id = zeitrak::invitation::invite_member(
        &workspace_id,
        &current_user,
        email,
        role_id,
        ttl_days,
        &*email_sender,
        base_url,
    )
    .await
    .map_err(internal)?;

    Ok(invitation_id.to_string())
}

/// Returns invitation details for the given token.
///
/// This endpoint is public (no session required) so that uninvited visitors can
/// view the workspace name before deciding to register.
#[server]
#[get("/api/invitations/by-token")]
pub async fn get_invitation_by_token(
    token: String,
) -> Result<Option<InvitationDto>, ServerFnError> {
    use crate::session::internal;
    use zeitrak::core::admin::invitation::InvitationStatus;

    tracing::info!(token = %token, "looking up invitation by token");

    let row = zeitrak::invitation::get_invitation_by_token(&token)
        .await
        .map_err(|e| {
            tracing::error!(token = %token, error = %e, "invitation lookup failed");
            internal(e)
        })?;

    match &row {
        Some(r) => tracing::info!(
            token = %token,
            invitation_id = %r.id(),
            status = ?r.status,
            email = %r.email(),
            "invitation found"
        ),
        None => tracing::warn!(token = %token, "no invitation found for token"),
    }

    Ok(row.map(|r| {
        let status = match r.status {
            InvitationStatus::Pending => "pending",
            InvitationStatus::Accepted => "accepted",
            InvitationStatus::Revoked => "revoked",
        };
        InvitationDto {
            id: r.id().to_string(),
            workspace_id: r.workspace_id.to_string(),
            workspace_name: r.workspace_name.clone(),
            email: r.email().to_string(),
            workspace_role_id: r.workspace_role_id.to_string(),
            token: r.token().to_string(),
            status: status.to_string(),
        }
    }))
}

/// Accepts the invitation identified by `token` for the currently authenticated user.
///
/// On success the user is assigned to the workspace and the session is updated to
/// select that workspace.
#[post("/api/invitations/accept")]
pub async fn accept_invitation(token: String) -> Result<String, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _accept_invitation(token).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = token;
        Ok(String::new())
    }
}

/// Returns all invitations for the current workspace.
#[get("/api/invitations")]
pub async fn list_invitations() -> Result<Vec<InvitationDto>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _list_invitations().await
    }
    #[cfg(not(feature = "server"))]
    {
        Ok(vec![])
    }
}

/// Returns all pending invitations addressed to the currently authenticated user.
#[get("/api/invitations/mine")]
pub async fn list_my_invitations() -> Result<Vec<InvitationDto>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _list_my_invitations().await
    }
    #[cfg(not(feature = "server"))]
    {
        Ok(vec![])
    }
}

/// Revokes a pending invitation by its token.
///
/// Requires the `member.invite` permission in the invitation's workspace.
#[post("/api/invitations/revoke")]
pub async fn revoke_invitation(token: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _revoke_invitation(token).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = token;
        Ok(())
    }
}

/// Declines a pending invitation addressed to the currently authenticated user.
#[post("/api/invitations/decline")]
pub async fn decline_invitation(token: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _decline_invitation(token).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = token;
        Ok(())
    }
}

/// Registers a new user and immediately accepts a pending invitation.
///
/// Used during invitation-only registration: the email is taken from the invitation,
/// so only name and password are required from the user.
/// On success a session is started with the workspace already selected.
#[post("/api/invitations/register-and-accept")]
pub async fn register_and_accept(
    name: String,
    password: String,
    token: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _register_and_accept(name, password, token).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (name, password, token);
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn _send_invitation(
    email: String,
    workspace_role_id: String,
    ttl_days: u32,
) -> Result<String, ServerFnError> {
    use crate::session::{internal, require_permission, session_workspace};

    let (user, workspace_id) = session_workspace().await?;
    require_permission(&user, zeitrak::core::permissions::MEMBER_INVITE).await?;

    let role_id: zeitrak::core::admin::workspace_role::WorkspaceRoleId = workspace_role_id
        .parse()
        .map_err(|_| ServerFnError::ServerError {
            message: "invalid workspace_role_id".into(),
            code: 400,
            details: None,
        })?;

    let email_sender = zeitrak::email::email_sender_from_config().await.map_err(internal)?;
    let base_url = zeitrak::email::base_url();

    let current_user = zeitrak::auth::CurrentUser {
        id: user.id,
        email: user.email,
    };

    let invitation_id = zeitrak::invitation::invite_member(
        &workspace_id,
        &current_user,
        email,
        role_id,
        ttl_days,
        &*email_sender,
        base_url,
    )
    .await
    .map_err(internal)?;

    Ok(invitation_id.to_string())
}

#[cfg(feature = "server")]
async fn _accept_invitation(token: String) -> Result<String, ServerFnError> {
    use crate::session::{internal, session_user};
    use dioxus::fullstack::extract;
    use tower_sessions::Session;

    let mut user = session_user().await?;

    let workspace_id = zeitrak::invitation::accept_invitation(&token, &user.id)
        .await
        .map_err(internal)?;

    user.workspace_id = Some(workspace_id.to_string());
    let session: Session = extract().await?;
    session
        .insert("user", user)
        .await
        .map_err(|e| ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        })?;

    Ok(workspace_id.to_string())
}

#[cfg(feature = "server")]
async fn _register_and_accept(
    name: String,
    password: String,
    token: String,
) -> Result<(), ServerFnError> {
    use crate::{auth::UserInfo, session::internal};
    use dioxus::fullstack::extract;
    use tower_sessions::Session;
    use zeitrak::core::admin::invitation::InvitationStatus;

    let invitation = zeitrak::invitation::get_invitation_by_token(&token)
        .await
        .map_err(internal)?
        .ok_or_else(|| ServerFnError::ServerError {
            message: "Invitation not found or has expired.".into(),
            code: 404,
            details: None,
        })?;

    if invitation.status != InvitationStatus::Pending {
        return Err(ServerFnError::ServerError {
            message: "This invitation has already been used or revoked.".into(),
            code: 410,
            details: None,
        });
    }

    let email = invitation.email().to_string();

    let email_sender = zeitrak::email::email_sender_from_config().await.map_err(internal)?;
    let base_url = zeitrak::email::base_url();
    let user_id = zeitrak::registration::register_user(
        name,
        email.clone(),
        password,
        &*email_sender,
        base_url,
    )
    .await
    .map_err(internal)?;

    let workspace_id = zeitrak::invitation::accept_invitation(&token, &user_id.to_string())
        .await
        .map_err(internal)?;

    let is_admin = zeitrak::authorization::AuthorizationService::is_admin(&user_id.to_string())
        .await
        .map_err(internal)?;

    let session: Session = extract().await?;
    session
        .insert(
            "user",
            UserInfo {
                id: user_id.to_string(),
                email,
                is_admin,
                workspace_id: Some(workspace_id.to_string()),
            },
        )
        .await
        .map_err(|e| ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        })
}

#[cfg(feature = "server")]
async fn _list_invitations() -> Result<Vec<InvitationDto>, ServerFnError> {
    use crate::session::{internal, session_workspace};
    use zeitrak::core::admin::invitation::InvitationStatus;

    let (_, workspace_id) = session_workspace().await?;
    let rows = zeitrak::invitation::list_workspace_invitations(&workspace_id)
        .await
        .map_err(internal)?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let status = match r.status {
                InvitationStatus::Pending => "pending",
                InvitationStatus::Accepted => "accepted",
                InvitationStatus::Revoked => "revoked",
            };
            InvitationDto {
                id: r.id().to_string(),
                workspace_id: r.workspace_id.to_string(),
                workspace_name: r.workspace_name.clone(),
                email: r.email().to_string(),
                workspace_role_id: r.workspace_role_id.to_string(),
                token: r.token().to_string(),
                status: status.to_string(),
            }
        })
        .collect())
}

#[cfg(feature = "server")]
async fn _list_my_invitations() -> Result<Vec<InvitationDto>, ServerFnError> {
    use crate::session::{internal, session_user};
    use zeitrak::core::admin::invitation::InvitationStatus;

    let user = session_user().await?;
    let rows = zeitrak::invitation::list_pending_invitations_for_email(&user.email)
        .await
        .map_err(internal)?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let status = match r.status {
                InvitationStatus::Pending => "pending",
                InvitationStatus::Accepted => "accepted",
                InvitationStatus::Revoked => "revoked",
            };
            InvitationDto {
                id: r.id().to_string(),
                workspace_id: r.workspace_id.to_string(),
                workspace_name: r.workspace_name.clone(),
                email: r.email().to_string(),
                workspace_role_id: r.workspace_role_id.to_string(),
                token: r.token().to_string(),
                status: status.to_string(),
            }
        })
        .collect())
}

#[cfg(feature = "server")]
async fn _revoke_invitation(token: String) -> Result<(), ServerFnError> {
    use crate::session::{internal, session_user};
    use zeitrak::auth::CurrentUser;

    let user = session_user().await?;
    let current_user = CurrentUser {
        id: user.id,
        email: user.email,
    };

    zeitrak::invitation::revoke_invitation(&token, &current_user)
        .await
        .map_err(internal)
}

#[cfg(feature = "server")]
async fn _decline_invitation(token: String) -> Result<(), ServerFnError> {
    use crate::session::{internal, session_user};

    let user = session_user().await?;
    zeitrak::invitation::decline_invitation(&token, &user.email)
        .await
        .map_err(internal)
}
