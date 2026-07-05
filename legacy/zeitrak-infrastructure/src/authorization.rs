use std::collections::HashSet;

use async_trait::async_trait;

/// Port: check and resolve permissions against whatever backing store the adapter provides.
///
/// Implementations live in `zeitrak-infrastructure-impl`. The trait is used directly by
/// `zeitrak-plugin-host` (which sits below the facade layer) to avoid a circular dependency.
#[async_trait]
pub trait AuthorizationRepository: Send + Sync {
    /// Returns `true` if the user holds the [`zeitrak_core::permissions::ADMIN_BYPASS`]
    /// permission through any workspace role.
    ///
    /// # Errors
    ///
    /// Returns [`AuthorizationError::Storage`] if the backing query fails.
    async fn is_admin(&self, user_id: &str) -> Result<bool, AuthorizationError>;

    /// Returns `true` if the user has `permission` in `workspace_id`, either through
    /// a workspace role or as a directly-granted individual permission.
    ///
    /// # Errors
    ///
    /// Returns [`AuthorizationError::Storage`] if the backing query fails.
    async fn has_permission(
        &self,
        user_id: &str,
        workspace_id: &str,
        permission: &str,
    ) -> Result<bool, AuthorizationError>;

    /// Returns the complete set of permission names held by `user_id` in `workspace_id`.
    ///
    /// Used by `zeitrak-plugin-host` to pre-populate [`PermissionSet`] at request time
    /// so that capability checks do not require async round-trips per plugin call.
    ///
    /// # Errors
    ///
    /// Returns [`AuthorizationError::Storage`] if the backing query fails.
    async fn user_permissions(
        &self,
        user_id: &str,
        workspace_id: &str,
    ) -> Result<HashSet<String>, AuthorizationError>;
}

/// Error returned by [`AuthorizationRepository`] methods.
///
/// Kept sqlx-free so that `zeitrak-infrastructure` does not need to depend on a
/// particular database driver.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AuthorizationError {
    /// The backing store returned an error.
    #[error("authorization query failed: {0}")]
    Storage(String),
}
