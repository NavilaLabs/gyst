use anyhow::{Result, bail};
use async_trait::async_trait;
use zeitrak_infrastructure_impl::Pool;
use sqlx::AnyPool;

use crate::authentication::CurrentUser;

// ── Policy trait ─────────────────────────────────────────────────────────────

/// Pluggable authorization strategy.
///
/// Implement this trait to replace or extend the default role-based logic
/// without touching the rest of the authorization service.  The pool is passed
/// on each call so that the same policy can be reused across admin and test
/// pools without holding state.
#[async_trait]
pub trait AuthorizationPolicy: Send + Sync {
    /// Returns `true` if `user_id` should be treated as a global admin.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    async fn is_admin(&self, pool: &AnyPool, user_id: &str) -> Result<bool>;

    /// Returns `true` if `user_id` has `permission` in `workspace_id`.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    async fn has_permission(
        &self,
        pool: &AnyPool,
        user_id: &str,
        workspace_id: &str,
        permission: &str,
    ) -> Result<bool>;
}

// ── Default implementation ────────────────────────────────────────────────────

/// Role-based policy backed by projection tables.
///
/// `admin_role_name` is the role name that grants unconditional access.
/// Override via [`RoleBasedPolicy::with_admin_role`] for custom setups.
pub struct RoleBasedPolicy {
    admin_role_name: &'static str,
}

impl Default for RoleBasedPolicy {
    fn default() -> Self {
        Self {
            admin_role_name: "admin",
        }
    }
}

impl RoleBasedPolicy {
    #[must_use]
    pub const fn with_admin_role(admin_role_name: &'static str) -> Self {
        Self { admin_role_name }
    }
}

#[async_trait]
impl AuthorizationPolicy for RoleBasedPolicy {
    async fn is_admin(&self, pool: &AnyPool, user_id: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM projections__workspace_user_roles wur
             JOIN projections__workspace_roles wr
               ON wur.workspace_role_id = wr.id
             WHERE wur.user_id = $1
               AND wr.name = $2",
        )
        .bind(user_id)
        .bind(self.admin_role_name)
        .fetch_one(pool)
        .await?;
        Ok(count > 0)
    }

    async fn has_permission(
        &self,
        pool: &AnyPool,
        user_id: &str,
        workspace_id: &str,
        permission: &str,
    ) -> Result<bool> {
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
}

// ── Service ───────────────────────────────────────────────────────────────────

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
    // ── internal helpers ──────────────────────────────────────────────────────

    async fn admin_pool() -> Result<AnyPool> {
        Ok(Pool::connect_admin().await?.into_pool())
    }

    fn policy() -> RoleBasedPolicy {
        RoleBasedPolicy::default()
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
        Self::policy().is_admin(pool, user_id).await
    }

    // ── has_permission ────────────────────────────────────────────────────────

    /// Returns `true` if the user has the named permission **in the given
    /// workspace**, either through a workspace role or a directly-granted
    /// individual permission.
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
        Self::policy()
            .has_permission(pool, user_id, workspace_id, permission)
            .await
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
