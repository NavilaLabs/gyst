use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// SMTP configuration data returned to the frontend.
///
/// Sensitive fields are masked — only their presence is indicated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmtpConfigFormDto {
    /// `"password"` or `"xoauth2"`.
    pub auth_method: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub from_address: String,
    pub use_tls: bool,
    /// `true` if a password is stored (never returned to the client).
    pub password_is_set: bool,
    pub client_id: Option<String>,
    pub tenant_id: Option<String>,
    /// `true` if a client secret is stored.
    pub client_secret_is_set: bool,
    pub oauth2_smtp_email: Option<String>,
    /// `true` once the Microsoft OAuth2 authorization flow has completed.
    pub oauth2_authorized: bool,
}

// ── Setup prefill (no auth — only available before setup is complete) ─────────

/// Returns the current SMTP configuration for the setup wizard.
///
/// This endpoint is intentionally unauthenticated because it is called during
/// first-time setup before any user exists.  It returns an error if setup has
/// already been completed.
#[server]
#[get("/api/setup/smtp-prefill")]
pub async fn smtp_prefill() -> Result<SmtpConfigFormDto, ServerFnError> {
    #[cfg(feature = "server")]
    {
        use crate::session::internal;

        // Only allow access before setup is complete.
        let is_complete = zeitrak::setup::is_setup_complete()
            .await
            .map_err(internal)?;
        if is_complete {
            return Err(ServerFnError::ServerError {
                message: "forbidden".into(),
                code: 403,
                details: None,
            });
        }

        let dto = zeitrak::smtp_config::get_smtp_config_dto()
            .await
            .map_err(internal)?;
        return Ok(facade_to_form_dto(dto));
    }
    #[cfg(not(feature = "server"))]
    Ok(SmtpConfigFormDto {
        auth_method: "password".into(),
        host: String::new(),
        port: 587,
        username: String::new(),
        from_address: String::new(),
        use_tls: true,
        password_is_set: false,
        client_id: None,
        tenant_id: None,
        client_secret_is_set: false,
        oauth2_smtp_email: None,
        oauth2_authorized: false,
    })
}

// ── Setup SMTP endpoints (no auth — only before setup is complete) ────────────

/// Saves the SMTP configuration during first-time setup.
///
/// Unauthenticated — returns an error if setup has already been completed.
#[server]
#[post("/api/setup/smtp-config")]
#[allow(clippy::too_many_arguments)]
pub async fn setup_save_smtp_config(
    auth_method: String,
    host: String,
    port: u16,
    username: String,
    from_address: String,
    use_tls: bool,
    password: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    tenant_id: Option<String>,
    oauth2_smtp_email: Option<String>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        use crate::session::internal;

        let is_complete = zeitrak::setup::is_setup_complete()
            .await
            .map_err(internal)?;
        if is_complete {
            return Err(ServerFnError::ServerError {
                message: "forbidden".into(),
                code: 403,
                details: None,
            });
        }

        return zeitrak::smtp_config::save_smtp_config(
            auth_method,
            host,
            port,
            username,
            from_address,
            use_tls,
            password,
            client_id,
            client_secret,
            tenant_id,
            oauth2_smtp_email,
        )
        .await
        .map_err(internal);
    }
    #[cfg(not(feature = "server"))]
    Ok(())
}

/// Initiates the Microsoft OAuth2 flow during first-time setup.
///
/// Unauthenticated — returns an error if setup has already been completed.
#[server]
#[post("/api/setup/smtp-oauth2/start")]
pub async fn setup_start_microsoft_oauth2(
    client_id: String,
    tenant_id: String,
) -> Result<String, ServerFnError> {
    #[cfg(feature = "server")]
    {
        use crate::session::internal;

        let is_complete = zeitrak::setup::is_setup_complete()
            .await
            .map_err(internal)?;
        if is_complete {
            return Err(ServerFnError::ServerError {
                message: "forbidden".into(),
                code: 403,
                details: None,
            });
        }

        return zeitrak::smtp_oauth2::initiate_microsoft_oauth2(&client_id, &tenant_id)
            .await
            .map_err(internal);
    }
    #[cfg(not(feature = "server"))]
    Ok(String::new())
}

/// Returns whether Microsoft OAuth2 authorization has completed during setup.
///
/// Unauthenticated — returns an error if setup has already been completed.
#[server]
#[get("/api/setup/smtp-oauth2/status")]
pub async fn setup_oauth2_status() -> Result<bool, ServerFnError> {
    #[cfg(feature = "server")]
    {
        use crate::session::internal;

        let is_complete = zeitrak::setup::is_setup_complete()
            .await
            .map_err(internal)?;
        if is_complete {
            return Err(ServerFnError::ServerError {
                message: "forbidden".into(),
                code: 403,
                details: None,
            });
        }

        return zeitrak::smtp_oauth2::oauth2_status()
            .await
            .map_err(internal);
    }
    #[cfg(not(feature = "server"))]
    Ok(false)
}

// ── Admin SMTP config endpoints ───────────────────────────────────────────────

/// Returns the current SMTP configuration (admin only).
#[server]
#[get("/api/smtp/config")]
pub async fn get_smtp_config() -> Result<SmtpConfigFormDto, ServerFnError> {
    #[cfg(feature = "server")]
    {
        use crate::session::{internal, session_user};

        let user = session_user().await?;
        if !user.is_admin {
            return Err(ServerFnError::ServerError {
                message: "forbidden".into(),
                code: 403,
                details: None,
            });
        }

        let dto = zeitrak::smtp_config::get_smtp_config_dto()
            .await
            .map_err(internal)?;
        return Ok(facade_to_form_dto(dto));
    }
    #[cfg(not(feature = "server"))]
    Ok(SmtpConfigFormDto {
        auth_method: "password".into(),
        host: String::new(),
        port: 587,
        username: String::new(),
        from_address: String::new(),
        use_tls: true,
        password_is_set: false,
        client_id: None,
        tenant_id: None,
        client_secret_is_set: false,
        oauth2_smtp_email: None,
        oauth2_authorized: false,
    })
}

/// Saves the SMTP configuration (admin only).
#[server]
#[post("/api/smtp/config")]
#[allow(clippy::too_many_arguments)]
pub async fn save_smtp_config(
    auth_method: String,
    host: String,
    port: u16,
    username: String,
    from_address: String,
    use_tls: bool,
    password: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    tenant_id: Option<String>,
    oauth2_smtp_email: Option<String>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        use crate::session::{internal, session_user};

        let user = session_user().await?;
        if !user.is_admin {
            return Err(ServerFnError::ServerError {
                message: "forbidden".into(),
                code: 403,
                details: None,
            });
        }

        zeitrak::smtp_config::save_smtp_config(
            auth_method,
            host,
            port,
            username,
            from_address,
            use_tls,
            password,
            client_id,
            client_secret,
            tenant_id,
            oauth2_smtp_email,
        )
        .await
        .map_err(internal)?;
        return Ok(());
    }
    #[cfg(not(feature = "server"))]
    Ok(())
}

/// Sends a test email to `to_address` using the current SMTP config (admin only).
#[server]
#[post("/api/smtp/test")]
pub async fn test_smtp_connection(to_address: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        use crate::session::{internal, session_user};

        let user = session_user().await?;
        if !user.is_admin {
            return Err(ServerFnError::ServerError {
                message: "forbidden".into(),
                code: 403,
                details: None,
            });
        }

        return zeitrak::smtp_config::test_smtp_connection(to_address)
            .await
            .map_err(internal);
    }
    #[cfg(not(feature = "server"))]
    Ok(())
}

// ── OAuth2 endpoints ──────────────────────────────────────────────────────────

/// Initiates the Microsoft OAuth2 flow.
///
/// Returns the authorization URL which the frontend should open in a new tab.
/// The current SMTP config row must already have been saved with `client_id`
/// and `tenant_id` before calling this.
#[server]
#[post("/api/smtp/oauth2/microsoft/start")]
pub async fn start_microsoft_oauth2(
    client_id: String,
    tenant_id: String,
) -> Result<String, ServerFnError> {
    #[cfg(feature = "server")]
    {
        use crate::session::{internal, session_user};

        let user = session_user().await?;
        if !user.is_admin {
            return Err(ServerFnError::ServerError {
                message: "forbidden".into(),
                code: 403,
                details: None,
            });
        }

        return zeitrak::smtp_oauth2::initiate_microsoft_oauth2(&client_id, &tenant_id)
            .await
            .map_err(internal);
    }
    #[cfg(not(feature = "server"))]
    Ok(String::new())
}

/// Polls whether the Microsoft OAuth2 authorization has completed (admin only).
#[server]
#[get("/api/smtp/oauth2/status")]
pub async fn oauth2_status() -> Result<bool, ServerFnError> {
    #[cfg(feature = "server")]
    {
        use crate::session::{internal, session_user};

        let user = session_user().await?;
        if !user.is_admin {
            return Err(ServerFnError::ServerError {
                message: "forbidden".into(),
                code: 403,
                details: None,
            });
        }

        return zeitrak::smtp_oauth2::oauth2_status()
            .await
            .map_err(internal);
    }
    #[cfg(not(feature = "server"))]
    Ok(false)
}

// ── helpers ───────────────────────────────────────────────────────────────────

#[cfg(feature = "server")]
fn facade_to_form_dto(dto: zeitrak::smtp_config::SmtpConfigDto) -> SmtpConfigFormDto {
    SmtpConfigFormDto {
        auth_method: dto.auth_method,
        host: dto.host,
        port: dto.port,
        username: dto.username,
        from_address: dto.from_address,
        use_tls: dto.use_tls,
        password_is_set: dto.password_is_set,
        client_id: dto.client_id,
        tenant_id: dto.tenant_id,
        client_secret_is_set: dto.client_secret_is_set,
        oauth2_smtp_email: dto.oauth2_smtp_email,
        oauth2_authorized: dto.oauth2_authorized,
    }
}
