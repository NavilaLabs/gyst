use dioxus::prelude::*;

/// Returns `true` if setup has already been completed (at least one user exists).
/// The GUI uses this to redirect away from `/setup` when not needed.
#[server]
#[get("/api/setup/complete")]
pub async fn is_setup_complete() -> Result<bool, ServerFnError> {
    zeitrak::setup::is_setup_complete()
        .await
        .map_err(|e| ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        })
}

/// Runs first-time setup: creates the admin user, workspace, and admin role.
/// Returns an error if setup has already been completed.
#[server]
#[post("/api/setup")]
pub async fn setup(
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
