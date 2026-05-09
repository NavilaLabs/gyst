use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// The authenticated user stored in the server-side session.
/// Passed to client code only via server functions — never a raw JWT.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub is_admin: bool,
    /// The workspace (tenant) this user belongs to. `None` for users with no
    /// workspace assignment (should not happen in a properly set-up instance).
    pub workspace_id: Option<String>,
}

/// Returns the currently authenticated user, or `None` if not logged in.
#[get("/api/auth/me")]
pub async fn get_current_user() -> Result<Option<UserInfo>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        _get_current_user().await
    }
    #[cfg(not(feature = "server"))]
    {
        Ok(None)
    }
}

#[cfg(feature = "server")]
async fn _get_current_user() -> Result<Option<UserInfo>, ServerFnError> {
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
    Ok(user)
}

/// Destroys the current session, logging the user out.
#[post("/api/auth/logout")]
pub async fn logout() -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _logout().await
    }
    #[cfg(not(feature = "server"))]
    {
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn _logout() -> Result<(), ServerFnError> {
    use dioxus::fullstack::extract;
    use tower_sessions::Session;

    let session: Session = extract().await?;
    session
        .flush()
        .await
        .map_err(|e| ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        })
}
