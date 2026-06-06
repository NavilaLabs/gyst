use std::collections::HashSet;

use async_trait::async_trait;
use sqlx::AnyPool;
use zeitrak_core::permissions;
use zeitrak_infrastructure::authorization::{AuthorizationError, AuthorizationRepository};

/// SQL-backed implementation of [`AuthorizationRepository`].
///
/// Accepts a raw [`AnyPool`] so that the `_on(pool, …)` test helpers in
/// `zeitrak` continue to work with isolated test pools without needing the
/// phantom-typed [`Pool<ScopeAdmin, StateConnected>`].
#[derive(Clone)]
pub struct SqlAuthorizationRepository {
    pool: AnyPool,
}

impl SqlAuthorizationRepository {
    /// Construct from any connected pool that talks to the admin database.
    #[must_use]
    pub const fn new(pool: AnyPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthorizationRepository for SqlAuthorizationRepository {
    async fn is_admin(&self, user_id: &str) -> Result<bool, AuthorizationError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM projections__workspace_user_roles wur
             JOIN projections__workspace_role_permissions wrp
               ON wur.workspace_role_id = wrp.workspace_role_id
             JOIN permissions p
               ON wrp.permission_id = p.id
             WHERE wur.user_id = $1
               AND p.name = $2",
        )
        .bind(user_id)
        .bind(permissions::ADMIN_BYPASS)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AuthorizationError::Storage(e.to_string()))?;
        Ok(count > 0)
    }

    async fn has_permission(
        &self,
        user_id: &str,
        workspace_id: &str,
        permission: &str,
    ) -> Result<bool, AuthorizationError> {
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
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AuthorizationError::Storage(e.to_string()))?;

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
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AuthorizationError::Storage(e.to_string()))?;

        Ok(direct > 0)
    }

    async fn user_permissions(
        &self,
        user_id: &str,
        workspace_id: &str,
    ) -> Result<HashSet<String>, AuthorizationError> {
        let via_role: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT p.name
             FROM projections__workspace_user_roles wur
             JOIN projections__workspace_role_permissions wrp
               ON wur.workspace_role_id = wrp.workspace_role_id
             JOIN permissions p
               ON wrp.permission_id = p.id
             WHERE wur.user_id = $1
               AND wur.workspace_id = $2",
        )
        .bind(user_id)
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AuthorizationError::Storage(e.to_string()))?;

        let direct: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT p.name
             FROM projections__workspace_user_permissions wup
             JOIN permissions p
               ON wup.permission_id = p.id
             WHERE wup.user_id = $1
               AND wup.workspace_id = $2",
        )
        .bind(user_id)
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AuthorizationError::Storage(e.to_string()))?;

        Ok(via_role.into_iter().chain(direct).collect())
    }
}
