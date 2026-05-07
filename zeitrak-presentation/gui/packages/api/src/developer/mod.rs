use dioxus::prelude::*;

/// Migrates the admin database. Admin-only.
#[post("/api/database/migrate")]
pub async fn migrate_database() -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _migrate_database().await
    }
    #[cfg(not(feature = "server"))]
    {
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn _migrate_database() -> Result<(), ServerFnError> {
    use dioxus::fullstack::extract;
    use zeitrak::{
        authorization::AuthorizationService,
        infrastructure::{
            database::{Initializer, SqliteInitializationStrategy},
            Pool,
        },
        Migrate,
    };
    use tower_sessions::Session;

    // Require admin — only admins may trigger migrations.
    let session: Session = extract().await?;
    let user: Option<crate::auth::UserInfo> =
        session
            .get("user")
            .await
            .map_err(|e| ServerFnError::ServerError {
                message: e.to_string(),
                code: 500,
                details: None,
            })?;

    let user = user.ok_or_else(|| ServerFnError::ServerError {
        message: "unauthorized".to_string(),
        code: 401,
        details: None,
    })?;

    let current_user = zeitrak::authentication::CurrentUser {
        id: user.id,
        email: user.email,
    };

    AuthorizationService::require_admin(&current_user)
        .await
        .map_err(|_| ServerFnError::ServerError {
            message: "forbidden".to_string(),
            code: 403,
            details: None,
        })?;

    let default_pool =
        Pool::connect_default()
            .await
            .map_err(|error| ServerFnError::ServerError {
                message: error.to_string(),
                code: 500,
                details: None,
            })?;
    let initializer = Initializer::new(SqliteInitializationStrategy);
    let _ = initializer
        .initialize_admin(&default_pool)
        .await
        .map_err(|error| ServerFnError::ServerError {
            message: error.to_string(),
            code: 500,
            details: None,
        })?;

    let pool = Pool::connect_admin()
        .await
        .map_err(|error| ServerFnError::ServerError {
            message: error.to_string(),
            code: 500,
            details: None,
        })?;
    let _ = pool
        .migrate_database()
        .await
        .map_err(|error| ServerFnError::ServerError {
            message: error.to_string(),
            code: 500,
            details: None,
        })?;

    Ok(())
}

/// Migrates the tenant database for the currently selected workspace. Admin-only.
#[post("/api/database/migrate-tenant")]
pub async fn migrate_tenant_database() -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        _migrate_tenant_database().await
    }
    #[cfg(not(feature = "server"))]
    {
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn _migrate_tenant_database() -> Result<(), ServerFnError> {
    use dioxus::fullstack::extract;
    use zeitrak::{
        authorization::AuthorizationService,
        infrastructure::{
            database::{Initializer, SqliteInitializationStrategy},
            Pool,
        },
        Migrate,
    };
    use tower_sessions::Session;

    let session: Session = extract().await?;
    let user: Option<crate::auth::UserInfo> =
        session
            .get("user")
            .await
            .map_err(|e| ServerFnError::ServerError {
                message: e.to_string(),
                code: 500,
                details: None,
            })?;

    let user = user.ok_or_else(|| ServerFnError::ServerError {
        message: "unauthorized".to_string(),
        code: 401,
        details: None,
    })?;

    let current_user = zeitrak::authentication::CurrentUser {
        id: user.id.clone(),
        email: user.email.clone(),
    };

    AuthorizationService::require_admin(&current_user)
        .await
        .map_err(|_| ServerFnError::ServerError {
            message: "forbidden".to_string(),
            code: 403,
            details: None,
        })?;

    let workspace_id = user
        .workspace_id
        .ok_or_else(|| ServerFnError::ServerError {
            message: "no workspace selected".to_string(),
            code: 400,
            details: None,
        })?;

    let default_pool = Pool::connect_default()
        .await
        .map_err(|e| ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        })?;

    Initializer::new(SqliteInitializationStrategy)
        .initialize_tenant(&default_pool, Some(&workspace_id))
        .await
        .map_err(|e| ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        })?;

    let tenant_pool =
        Pool::connect_tenant(&workspace_id)
            .await
            .map_err(|e| ServerFnError::ServerError {
                message: e.to_string(),
                code: 500,
                details: None,
            })?;

    tenant_pool
        .migrate_database()
        .await
        .map_err(|e| ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        })?;

    Ok(())
}
