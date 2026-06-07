use std::collections::HashSet;
use std::sync::Arc;

use zeitrak_core::admin::{user::UserId, workspace::WorkspaceId};
use zeitrak_core::shared::clock::Clock;
use zeitrak_infrastructure::authorization::AuthorizationRepository;

use crate::trust::ZeitrakTrustTier;

/// Pre-resolved set of permission names for a (user, workspace) pair.
///
/// Built at request time by calling [`AuthorizationRepository::user_permissions`]
/// so that capability checks in plugin call-paths do not require async DB
/// round-trips per plugin call.
#[derive(Debug, Clone, Default)]
pub struct PermissionSet(HashSet<String>);

impl PermissionSet {
    /// Wrap a resolved set of permission names.
    #[must_use]
    pub const fn new(permissions: HashSet<String>) -> Self {
        Self(permissions)
    }

    /// Returns `true` if the set contains `permission`.
    #[must_use]
    pub fn contains(&self, permission: &str) -> bool {
        self.0.contains(permission)
    }

    /// Returns the number of permissions in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Per-call host context threaded into every `runtime.call_plugin(...)` call.
///
/// The instance is constructed once per inbound request (HTTP, server-function,
/// projection run) by the `zeitrak` facade crate and passed through every plugin
/// dispatch so that policy callbacks have access to the calling user's identity
/// and resolved permissions without further async work.
///
/// # Construction
///
/// `zeitrak` constructs this from the authenticated session:
///
/// ```rust,ignore
/// let permissions = authz
///     .user_permissions(&user.id, &workspace_id)
///     .await?;
///
/// let ctx = ZeitrakHostCtx {
///     user_id:      Some(user_id),
///     workspace_id: Some(workspace_id),
///     permissions:  Arc::new(PermissionSet::new(permissions)),
///     trust_tier:   ZeitrakTrustTier::Tenant,
///     authz:        Arc::clone(&authz),
///     clock:        Arc::clone(&clock),
/// };
/// ```
///
/// Tests construct the struct directly with explicit fields.
#[derive(Clone)]
pub struct ZeitrakHostCtx {
    /// Authenticated user, if any (absent on anonymous/machine calls).
    pub user_id: Option<UserId>,
    /// Active workspace for this call.
    pub workspace_id: Option<WorkspaceId>,
    /// Pre-resolved permissions for `(user_id, workspace_id)`.
    pub permissions: Arc<PermissionSet>,
    /// Trust tier of the plugin being called (set per-plugin by the runtime).
    pub trust_tier: ZeitrakTrustTier,
    /// Live authorization repository for dynamic checks beyond the cached set.
    pub authz: Arc<dyn AuthorizationRepository>,
    /// Testable wall-clock.
    pub clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for ZeitrakHostCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZeitrakHostCtx")
            .field("user_id", &self.user_id)
            .field("workspace_id", &self.workspace_id)
            .field("trust_tier", &self.trust_tier)
            .finish_non_exhaustive()
    }
}
