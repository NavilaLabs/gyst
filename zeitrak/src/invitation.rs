use anyhow::Result;
use zeitrak_core::admin::user::{UserQuery, UserQueryTrait};
use zeitrak_core::admin::{
    invitation::{
        InvitationCommand, InvitationCommandTrait, InvitationId, InvitationQuery,
        InvitationQueryTrait, InvitationRow,
    },
    workspace::{
        WorkspaceCommand, WorkspaceCommandTrait, WorkspaceId, WorkspaceQuery, WorkspaceQueryTrait,
    },
    workspace_role::WorkspaceRoleId,
};
use zeitrak_infrastructure::email::EmailSender;
use zeitrak_infrastructure_impl::{
    Pool,
    admin::{
        invitation::repositories::InvitationRepository, user::repositories::UserRepository,
        workspace::repositories::WorkspaceRepository,
    },
};

use crate::{authentication::CurrentUser, authorization::AuthorizationService};
use zeitrak_core::permissions;

/// Creates a workspace invitation and sends an email to the invitee.
///
/// Requires the `member.invite` permission in the given workspace.
///
/// # Errors
///
/// Returns a 403-style error if the user lacks permission, or a domain error
/// on invalid input or email delivery failure.
pub async fn invite_member(
    workspace_id: &str,
    invited_by: &CurrentUser,
    email: String,
    workspace_role_id: WorkspaceRoleId,
    ttl_days: u32,
    email_sender: &dyn EmailSender,
    base_url: &str,
) -> Result<InvitationId> {
    AuthorizationService::require_permission(invited_by, workspace_id, permissions::MEMBER_INVITE)
        .await?;

    let pool = Pool::connect_admin().await?;
    let repo = InvitationRepository::from_pool(pool.clone()).await?;

    let workspace_id_parsed: WorkspaceId = workspace_id.parse()?;
    let invited_by_id = invited_by.id.parse()?;
    let invitation_id = InvitationId::new();

    let root = InvitationCommand::new(repo)
        .create(
            invitation_id.clone(),
            workspace_id_parsed,
            invited_by_id,
            email.clone(),
            workspace_role_id,
            ttl_days,
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let token = root.token().to_string();

    // The projector is async, so the projection row may not yet exist immediately
    // after the event is written. Poll until the row appears before sending the
    // email — this guarantees the link is live when the recipient clicks it.
    let projection_ready = {
        let pool = Pool::connect_admin().await?;
        let mut ready = false;
        for _ in 0..20u8 {
            let repo = InvitationRepository::from_pool(pool.clone()).await?;
            if InvitationQuery::new(repo)
                .find_by_token(&token)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?
                .is_some()
            {
                ready = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        ready
    };
    anyhow::ensure!(
        projection_ready,
        "invitation projection was not populated within 2 s — the projection daemon may not be running"
    );

    let link = format!("{base_url}/invitations/accept/{token}");

    let workspace_name = WorkspaceQuery::new(WorkspaceRepository::from_pool(pool.clone()).await?)
        .find_view_by_id(workspace_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .map_or_else(
            || "Zeitrak".to_string(),
            |w| w.name().unwrap_or("Zeitrak").to_string(),
        );

    let inviter_name = UserQuery::new(UserRepository::from_pool(pool).await?)
        .find_view_by_id(&invited_by.id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .map_or_else(|| invited_by.email.clone(), |u| u.name().to_string());

    email_sender
        .send_invitation(&email, &link, &workspace_name, &inviter_name, ttl_days)
        .await?;

    Ok(invitation_id)
}

/// Returns the invitation associated with the given token, or `None` if not found.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub async fn get_invitation_by_token(token: &str) -> Result<Option<InvitationRow>> {
    tracing::debug!(token = %token, "querying invitation projection by token");
    let pool = Pool::connect_admin().await?;
    let repo = InvitationRepository::from_pool(pool).await?;
    let result = InvitationQuery::new(repo)
        .find_by_token(token)
        .await
        .map_err(|e| {
            tracing::error!(token = %token, error = %e, "find_by_token query failed");
            anyhow::anyhow!("{e}")
        })?;
    if let Some(row) = &result {
        tracing::debug!(
            token = %token,
            invitation_id = %row.id(),
            status = ?row.status,
            expires_at = %row.expires_at,
            "invitation row found in projection"
        );
    } else {
        tracing::warn!(token = %token, "invitation not found in projections__invitations");
    }
    Ok(result)
}

/// Returns all invitations for the given workspace.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub async fn list_workspace_invitations(workspace_id: &str) -> Result<Vec<InvitationRow>> {
    let pool = Pool::connect_admin().await?;
    let repo = InvitationRepository::from_pool(pool).await?;
    let ws_id: WorkspaceId = workspace_id.parse()?;
    InvitationQuery::new(repo)
        .find_by_workspace(&ws_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Returns all pending, non-expired invitations addressed to the given email.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub async fn list_pending_invitations_for_email(email: &str) -> Result<Vec<InvitationRow>> {
    let pool = Pool::connect_admin().await?;
    let repo = InvitationRepository::from_pool(pool).await?;
    InvitationQuery::new(repo)
        .find_all_pending_for_email(email)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Revokes a pending invitation on behalf of a workspace admin.
///
/// Requires the `member.invite` permission in the invitation's workspace.
///
/// # Errors
///
/// Returns an error if the token is invalid, the caller lacks permission, or
/// the database operation fails.
pub async fn revoke_invitation(token: &str, revoked_by: &CurrentUser) -> Result<()> {
    let pool = Pool::connect_admin().await?;
    let repo = InvitationRepository::from_pool(pool.clone()).await?;

    let row = InvitationQuery::new(repo)
        .find_by_token(token)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .ok_or_else(|| anyhow::anyhow!("invitation not found"))?;

    AuthorizationService::require_permission(
        revoked_by,
        &row.workspace_id.to_string(),
        permissions::MEMBER_INVITE,
    )
    .await?;

    InvitationCommand::new(InvitationRepository::from_pool(pool).await?)
        .revoke(row.id().clone())
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Declines a pending invitation on behalf of the invited user.
///
/// Verifies that the invitation email matches `user_email` before revoking.
///
/// # Errors
///
/// Returns an error if the token is invalid, the email does not match, or the
/// database operation fails.
pub async fn decline_invitation(token: &str, user_email: &str) -> Result<()> {
    let pool = Pool::connect_admin().await?;
    let repo = InvitationRepository::from_pool(pool.clone()).await?;

    let row = InvitationQuery::new(repo)
        .find_by_token(token)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .ok_or_else(|| anyhow::anyhow!("invitation not found"))?;

    anyhow::ensure!(
        row.email() == user_email,
        "invitation does not belong to this user"
    );

    InvitationCommand::new(InvitationRepository::from_pool(pool).await?)
        .revoke(row.id().clone())
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Accepts a workspace invitation on behalf of the authenticated user.
///
/// On success the user is assigned to the workspace with the role specified in
/// the invitation.  No new tenant database is created — the workspace already
/// has one.
///
/// # Errors
///
/// Returns an error if the token is invalid, the invitation is expired/already
/// used, or the assignment fails.
pub async fn accept_invitation(token: &str, user_id: &str) -> Result<WorkspaceId> {
    let pool = Pool::connect_admin().await?;
    let repo = InvitationRepository::from_pool(pool.clone()).await?;

    let row = InvitationQuery::new(repo)
        .find_by_token(token)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .ok_or_else(|| anyhow::anyhow!("invitation not found"))?;

    let invitation_id = row.id().clone();
    let workspace_id = row.workspace_id.clone();
    let workspace_role_id = row.workspace_role_id.clone();

    let accepted_by = user_id.parse()?;
    InvitationCommand::new(InvitationRepository::from_pool(pool.clone()).await?)
        .accept(invitation_id, accepted_by)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let user_id_parsed = user_id.parse()?;
    WorkspaceCommand::new(WorkspaceRepository::from_pool(pool).await?)
        .assign_user_role(workspace_id.clone(), user_id_parsed, workspace_role_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(workspace_id)
}
