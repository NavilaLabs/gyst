use anyhow::{Result, bail};
use loom_infrastructure_impl::Pool;
use sqlx::AnyPool;

use crate::authentication::CurrentUser;

/// Live permission checks against the projection tables.
///
/// ## Production API (static methods)
///
/// The static methods (`is_admin`, `has_permission`, `require_admin`,
/// `require_permission`) each open a fresh connection from the shared admin
/// pool, run a single parameterised SQL query, and return immediately — they
/// never touch the event store.  Use these in server functions and middleware.
///
/// ## Test API (`_on` methods)
///
/// The `_on` counterparts accept a `&AnyPool` argument so that tests can pass
/// an isolated in-memory pool instead of relying on the global `CONFIG`-driven
/// pool.  This makes each test fully self-contained and allows them to run in
/// parallel without `#[serial]`.
pub struct AuthorizationService;

impl AuthorizationService {
    // ── internal helper ───────────────────────────────────────────────────────

    async fn admin_pool() -> Result<AnyPool> {
        Ok(Pool::connect_admin().await?.into_pool())
    }

    // ── is_admin ──────────────────────────────────────────────────────────────

    /// Returns `true` if the user holds an "admin" role in any workspace.
    ///
    /// Admins implicitly have every permission; call this before any
    /// fine-grained [`has_permission`] check to implement a short-circuit.
    pub async fn is_admin(user_id: &str) -> Result<bool> {
        Self::is_admin_on(&Self::admin_pool().await?, user_id).await
    }

    /// Pool-injected version of [`is_admin`] — use this in tests.
    pub async fn is_admin_on(pool: &AnyPool, user_id: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM projections__workspace_user_roles wur
             JOIN projections__workspace_roles wr
               ON wur.workspace_role_id = wr.id
             WHERE wur.user_id = $1
               AND wr.name = 'admin'",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        Ok(count > 0)
    }

    // ── has_permission ────────────────────────────────────────────────────────

    /// Returns `true` if the user has the named permission **in the given
    /// workspace**, either through a workspace role or a directly-granted
    /// individual permission.
    ///
    /// The `workspace_id` parameter scopes the check to a single tenant so
    /// that a permission granted in workspace A never bleeds into workspace B.
    pub async fn has_permission(
        user_id: &str,
        workspace_id: &str,
        permission: &str,
    ) -> Result<bool> {
        Self::has_permission_on(
            &Self::admin_pool().await?,
            user_id,
            workspace_id,
            permission,
        )
        .await
    }

    /// Pool-injected version of [`has_permission`] — use this in tests.
    pub async fn has_permission_on(
        pool: &AnyPool,
        user_id: &str,
        workspace_id: &str,
        permission: &str,
    ) -> Result<bool> {
        // Check via role-level grant first.
        let via_role: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM projections__workspace_user_roles wur
             JOIN projections__workspace_role_permissions wrp
               ON wur.workspace_role_id = wrp.workspace_role_id
             JOIN permissions p
               ON wrp.permission_id = p.id
             WHERE wur.user_id = $1
               AND wur.workspace_id = $2
               AND p.name = $3",
        )
        .bind(user_id)
        .bind(workspace_id)
        .bind(permission)
        .fetch_one(pool)
        .await?;

        if via_role > 0 {
            return Ok(true);
        }

        // Fall back to a direct per-user grant.
        let direct: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM projections__workspace_user_permissions wup
             JOIN permissions p
               ON wup.permission_id = p.id
             WHERE wup.user_id = $1
               AND wup.workspace_id = $2
               AND p.name = $3",
        )
        .bind(user_id)
        .bind(workspace_id)
        .bind(permission)
        .fetch_one(pool)
        .await?;

        Ok(direct > 0)
    }

    // ── require_admin ─────────────────────────────────────────────────────────

    /// Require that the requesting user is an admin, returning a generic
    /// "forbidden" error if they are not.
    ///
    /// # Errors
    ///
    /// Returns an error if the admin pool cannot be obtained or the query fails.
    pub async fn require_admin(user: &CurrentUser) -> Result<()> {
        Self::require_admin_on(&Self::admin_pool().await?, user).await
    }

    /// Pool-injected version of [`require_admin`] — use this in tests.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails or the user is not an admin.
    pub async fn require_admin_on(pool: &AnyPool, user: &CurrentUser) -> Result<()> {
        if Self::is_admin_on(pool, &user.id).await? {
            Ok(())
        } else {
            bail!("forbidden")
        }
    }

    // ── require_permission ────────────────────────────────────────────────────

    /// Require that the requesting user has the named permission in the given
    /// workspace (or is a global admin), returning a generic "forbidden" error
    /// otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if the admin pool cannot be obtained, the query fails, or the user lacks permission.
    pub async fn require_permission(
        user: &CurrentUser,
        workspace_id: &str,
        permission: &str,
    ) -> Result<()> {
        Self::require_permission_on(&Self::admin_pool().await?, user, workspace_id, permission)
            .await
    }

    /// Pool-injected version of [`require_permission`] — use this in tests.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails or the user lacks the required permission.
    pub async fn require_permission_on(
        pool: &AnyPool,
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
