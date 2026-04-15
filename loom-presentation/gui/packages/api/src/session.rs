//! Server-side session extraction helpers shared across all API modules.
//!
//! All functions are `#[cfg(feature = "server")]` — this module compiles
//! to an empty stub on the client (WASM) side.

#[cfg(feature = "server")]
use dioxus::prelude::ServerFnError;

/// Extract the currently authenticated user from the session.
///
/// Returns a 401 error when the session contains no user (not logged in).
#[cfg(feature = "server")]
pub async fn session_user() -> Result<crate::auth::UserInfo, ServerFnError> {
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
    user.ok_or_else(|| ServerFnError::ServerError {
        message: "not authenticated".into(),
        code: 401,
        details: None,
    })
}

/// Extract the user's current workspace ID from the session.
///
/// Returns a 401 error when no workspace is selected.
#[cfg(feature = "server")]
pub async fn session_workspace() -> Result<(crate::auth::UserInfo, String), ServerFnError> {
    let user = session_user().await?;
    let workspace_id = user
        .workspace_id
        .clone()
        .ok_or_else(|| ServerFnError::ServerError {
            message: "no workspace selected".into(),
            code: 401,
            details: None,
        })?;
    Ok((user, workspace_id))
}

/// Require that the session user holds the named permission in their current
/// workspace.
///
/// Admins (any user assigned the "admin" workspace role) implicitly pass every
/// check. Returns 401 when no workspace is selected, 403 when the user lacks
/// the permission.
#[cfg(feature = "server")]
pub async fn require_permission(
    user: &crate::auth::UserInfo,
    permission: &str,
) -> Result<(), ServerFnError> {
    use loom::authentication::CurrentUser;
    use loom::authorization::AuthorizationService;

    let workspace_id = user
        .workspace_id
        .as_deref()
        .ok_or_else(|| ServerFnError::ServerError {
            message: "no workspace selected".into(),
            code: 401,
            details: None,
        })?;

    let current_user = CurrentUser {
        id: user.id.clone(),
        email: user.email.clone(),
    };
    AuthorizationService::require_permission(&current_user, workspace_id, permission)
        .await
        .map_err(|_| ServerFnError::ServerError {
            message: "forbidden".into(),
            code: 403,
            details: None,
        })
}

/// Map an `anyhow::Error` to a `ServerFnError`.
///
/// Returns 422 Unprocessable Entity when the error is a `loom::error::ValidationError`
/// (domain-level input validation), and 500 Internal Server Error for everything else.
#[cfg(feature = "server")]
pub fn internal(e: anyhow::Error) -> ServerFnError {
    if let Some(ve) = e.downcast_ref::<loom::error::ValidationError>() {
        return ServerFnError::ServerError {
            message: ve.to_string(),
            code: 422,
            details: None,
        };
    }
    ServerFnError::ServerError {
        message: e.to_string(),
        code: 500,
        details: None,
    }
}
