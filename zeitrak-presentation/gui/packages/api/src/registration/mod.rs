use dioxus::prelude::*;

/// Returns `true` when registration requires an invitation.
#[get("/api/registration/is-invite-only")]
pub async fn is_invite_only() -> Result<bool, ServerFnError> {
    #[cfg(feature = "server")]
    {
        Ok(zeitrak::registration::is_invite_only())
    }
    #[cfg(not(feature = "server"))]
    {
        Ok(false)
    }
}

/// Verifies a user's email address using the one-time token from the verification link.
///
/// Returns `()` on success. The session is not modified — the user still needs to log in.
#[server]
#[post("/api/verify-email")]
pub async fn verify_email(token: String) -> Result<(), ServerFnError> {
    use crate::session::internal;
    zeitrak::registration::verify_email_by_token(&token)
        .await
        .map(|_| ())
        .map_err(internal)
}

/// Registers a new user account.
///
/// Returns `()` on success. On success, a session is started (no workspace
/// selected yet — the user must create or accept an invitation to one).
/// Returns a 403 error when the server is configured for invitation-only registration.
#[post("/api/register")]
pub async fn register(name: String, email: String, password: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        if zeitrak::registration::is_invite_only() {
            return Err(ServerFnError::ServerError {
                message: "Registration is by invitation only.".into(),
                code: 403,
                details: None,
            });
        }
        _register(name, email, password).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (name, email, password);
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn _register(name: String, email: String, password: String) -> Result<(), ServerFnError> {
    use crate::{auth::UserInfo, session::internal};
    use dioxus::fullstack::extract;
    use tower_sessions::Session;

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
                can_manage_workspace: false,
                workspace_id: None,
            },
        )
        .await
        .map_err(|e| ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        })
}
