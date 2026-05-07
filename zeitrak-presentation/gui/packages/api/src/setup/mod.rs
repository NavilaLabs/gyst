use dioxus::prelude::*;

/// Returns `true` if setup has already been completed (at least one user exists).
/// The GUI uses this to redirect away from `/setup` when not needed.
#[get("/api/setup/complete")]
pub async fn is_setup_complete() -> Result<bool, ServerFnError> {
    #[cfg(feature = "server")]
    {
        zeitrak::setup::is_setup_complete()
            .await
            .map_err(|e| ServerFnError::ServerError {
                message: e.to_string(),
                code: 500,
                details: None,
            })
    }
    #[cfg(not(feature = "server"))]
    {
        Ok(false)
    }
}

/// Runs first-time setup: creates the admin user, workspace, and admin role.
/// Returns an error if setup has already been completed.
#[post("/api/setup")]
pub async fn setup(
    username: String,
    email: String,
    password: String,
    workspace_name: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _setup(username, email, password, workspace_name).await
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (username, email, password, workspace_name);
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn _setup(
    username: String,
    email: String,
    password: String,
    workspace_name: String,
) -> Result<(), ServerFnError> {
    zeitrak::setup::setup_application(username, email, password, workspace_name)
        .await
        .map_err(|e| ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        })
}
