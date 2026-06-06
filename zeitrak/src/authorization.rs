use std::collections::HashSet;
use std::sync::Arc;

use anyhow::{Result, bail};
use zeitrak_infrastructure::authorization::AuthorizationRepository;
use zeitrak_infrastructure_impl::{Pool, SqlAuthorizationRepository};

use crate::authentication::CurrentUser;

// ── Service ───────────────────────────────────────────────────────────────────

/// Live permission checks against the admin projection tables.
///
/// ## Production API (static methods)
///
/// The static methods (`is_admin`, `has_permission`, `require_admin`,
/// `require_permission`) each acquire a fresh pool from global config, delegate
/// to [`SqlAuthorizationRepository`], and return immediately — they never touch
/// the event store.  Use these in server functions and middleware.
///
/// ## Test API (`_on` methods)
///
/// The `_on` counterparts accept a `&sqlx::AnyPool` argument so that tests can
/// pass an isolated pool instead of relying on the global config-driven pool.
/// This makes each test fully self-contained and allows them to run in parallel
/// without `#[serial]`.
pub struct AuthorizationService;

impl AuthorizationService {
    // ── internal helpers ──────────────────────────────────────────────────────

    async fn repo() -> Result<SqlAuthorizationRepository> {
        Ok(SqlAuthorizationRepository::new(
            Pool::connect_admin().await?.into_pool(),
        ))
    }

    fn repo_on(pool: &sqlx::AnyPool) -> SqlAuthorizationRepository {
        SqlAuthorizationRepository::new(pool.clone())
    }

    // ── is_admin ──────────────────────────────────────────────────────────────

    /// Returns `true` if the user holds the admin-bypass permission in any workspace.
    ///
    /// # Errors
    ///
    /// Returns an error if the pool cannot be acquired or the query fails.
    pub async fn is_admin(user_id: &str) -> Result<bool> {
        Ok(Self::repo().await?.is_admin(user_id).await?)
    }

    /// Pool-injected version of [`is_admin`] — use in tests.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn is_admin_on(pool: &sqlx::AnyPool, user_id: &str) -> Result<bool> {
        Ok(Self::repo_on(pool).is_admin(user_id).await?)
    }

    // ── has_permission ────────────────────────────────────────────────────────

    /// Returns `true` if the user has `permission` in `workspace_id`.
    ///
    /// # Errors
    ///
    /// Returns an error if the pool cannot be acquired or the query fails.
    pub async fn has_permission(
        user_id: &str,
        workspace_id: &str,
        permission: &str,
    ) -> Result<bool> {
        Ok(Self::repo()
            .await?
            .has_permission(user_id, workspace_id, permission)
            .await?)
    }

    /// Pool-injected version of [`has_permission`] — use in tests.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn has_permission_on(
        pool: &sqlx::AnyPool,
        user_id: &str,
        workspace_id: &str,
        permission: &str,
    ) -> Result<bool> {
        Ok(Self::repo_on(pool)
            .has_permission(user_id, workspace_id, permission)
            .await?)
    }

    // ── user_permissions ──────────────────────────────────────────────────────

    /// Returns all permissions held by `user_id` in `workspace_id`.
    ///
    /// # Errors
    ///
    /// Returns an error if the pool cannot be acquired or the query fails.
    pub async fn user_permissions(
        user_id: &str,
        workspace_id: &str,
    ) -> Result<HashSet<String>> {
        Ok(Self::repo()
            .await?
            .user_permissions(user_id, workspace_id)
            .await?)
    }

    /// Pool-injected version of [`user_permissions`] — use in tests.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn user_permissions_on(
        pool: &sqlx::AnyPool,
        user_id: &str,
        workspace_id: &str,
    ) -> Result<HashSet<String>> {
        Ok(Self::repo_on(pool)
            .user_permissions(user_id, workspace_id)
            .await?)
    }

    // ── require_admin ─────────────────────────────────────────────────────────

    /// Require that the requesting user is an admin.
    ///
    /// # Errors
    ///
    /// Returns an error if the pool cannot be acquired, the query fails, or the
    /// user is not an admin.
    pub async fn require_admin(user: &CurrentUser) -> Result<()> {
        if Self::is_admin(&user.id).await? {
            Ok(())
        } else {
            bail!("forbidden")
        }
    }

    /// Pool-injected version of [`require_admin`] — use in tests.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails or the user is not an admin.
    pub async fn require_admin_on(pool: &sqlx::AnyPool, user: &CurrentUser) -> Result<()> {
        if Self::is_admin_on(pool, &user.id).await? {
            Ok(())
        } else {
            bail!("forbidden")
        }
    }

    // ── require_permission ────────────────────────────────────────────────────

    /// Require that the requesting user has `permission` in `workspace_id` (or
    /// is a global admin).
    ///
    /// # Errors
    ///
    /// Returns an error if the pool cannot be acquired, the query fails, or the
    /// user lacks permission.
    pub async fn require_permission(
        user: &CurrentUser,
        workspace_id: &str,
        permission: &str,
    ) -> Result<()> {
        let repo = Self::repo().await?;
        if repo.is_admin(&user.id).await? {
            return Ok(());
        }
        if repo.has_permission(&user.id, workspace_id, permission).await? {
            Ok(())
        } else {
            bail!("forbidden")
        }
    }

    /// Pool-injected version of [`require_permission`] — use in tests.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails or the user lacks the required permission.
    pub async fn require_permission_on(
        pool: &sqlx::AnyPool,
        user: &CurrentUser,
        workspace_id: &str,
        permission: &str,
    ) -> Result<()> {
        if Self::is_admin_on(pool, &user.id).await? {
            return Ok(());
        }
        if Self::has_permission_on(pool, &user.id, workspace_id, permission).await? {
            Ok(())
        } else {
            bail!("forbidden")
        }
    }
}

// ── Arc<dyn AuthorizationRepository> helper ───────────────────────────────────

/// Convenience constructor for tests and application setup.
///
/// Returns a heap-allocated [`SqlAuthorizationRepository`] wrapped in `Arc`
/// and type-erased to `dyn AuthorizationRepository`.
#[must_use]
pub fn arc_authz(pool: sqlx::AnyPool) -> Arc<dyn AuthorizationRepository> {
    Arc::new(SqlAuthorizationRepository::new(pool))
}
