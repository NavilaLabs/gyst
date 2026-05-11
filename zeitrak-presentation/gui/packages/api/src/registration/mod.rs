use dioxus::prelude::*;

/// Registers a new user account.
///
/// Returns `()` on success. On success, a session is started (no workspace
/// selected yet — the user must create or accept an invitation to one).
#[post("/api/register")]
pub async fn register(
    name: String,
    email: String,
    password: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
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

    let user_id = zeitrak::registration::register_user(name, email.clone(), password)
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
