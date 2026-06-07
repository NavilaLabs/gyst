use dioxus::prelude::*;

/// Validates credentials and creates a server-side session.
/// Returns `()` on success — no token is ever sent to the client.
#[post("/api/login")]
pub async fn login(email: String, password: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _login(email, password).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (email, password);
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn _login(email: String, password: String) -> Result<(), ServerFnError> {
    use crate::auth::UserInfo;
    use dioxus::fullstack::extract;
    use tower_sessions::Session;

    let token = zeitrak::authentication::login_user(&email, &password)
        .await
        .map_err(|e| ServerFnError::ServerError {
            message: e.to_string(),
            code: 401,
            details: None,
        })?;

    let current_user = zeitrak::authentication::validate_token(&token).map_err(|e| {
        ServerFnError::ServerError {
            message: e.to_string(),
            code: 401,
            details: None,
        }
    })?;

    let is_admin = zeitrak::authorization::AuthorizationService::is_admin(&current_user.id)
        .await
        .map_err(|e| ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        })?;

    let workspace_id = None; // User selects workspace after login.

    let session: Session = extract().await?;
    session
        .insert(
            "user",
            UserInfo {
                id: current_user.id,
                email: current_user.email,
                is_admin,
                can_manage_workspace: false,
                workspace_id,
            },
        )
        .await
        .map_err(|e| ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        })
}
