use dioxus_extism_host::CapabilityCheckFn;
use dioxus_extism_protocol::PluginId;

use crate::{host_ctx::ZeitrakHostCtx, trust::ZeitrakTrustTier};

/// Returns `true` if `permission` requires Instance or Signed-Instance trust
/// tier to hold.
///
/// Admin-scope permissions are those whose name starts with `"admin."`, matching
/// the `admin.*` namespace used by [`zeitrak_core::permissions`].
#[must_use]
pub fn is_admin_permission(permission: &str) -> bool {
    permission.starts_with("admin.")
}

/// Builds the `zeitrak.permission` capability check function.
///
/// The returned closure is registered with the `dioxus-extism` runtime under the
/// `"zeitrak.permission"` namespace.  It enforces two rules at plugin call time:
///
/// 1. **Admin-scope gate** — Permissions whose name starts with `"admin."` may only
///    be held by [`ZeitrakTrustTier::Instance`] or [`ZeitrakTrustTier::SignedInstance`]
///    plugins.
/// 2. **Runtime permission check** — The calling user's pre-resolved
///    [`PermissionSet`][crate::host_ctx::PermissionSet] must contain the permission.
///
/// Default-deny: any permission not in the set is rejected.
///
/// # Example — registering with the builder
///
/// ```rust,ignore
/// use zeitrak_plugin_host::capabilities::build_permission_capability_check;
///
/// let runtime = PluginRuntime::builder()
///     .with_capability_check_ctx("zeitrak.permission", build_permission_capability_check())
///     .build()
///     .await?;
/// ```
pub fn build_permission_capability_check() -> impl Fn(
    &PluginId,
    &serde_json::Value,
    &dioxus_extism_protocol::CallContext<'_, ZeitrakHostCtx>,
) -> Result<(), String>
+ Send
+ Sync
+ 'static {
    |_plugin_id, value, ctx| {
        let permission = value
            .as_str()
            .ok_or_else(|| "zeitrak.permission capability value must be a string".to_string())?;

        if is_admin_permission(permission) && ctx.host.trust_tier == ZeitrakTrustTier::Tenant {
            return Err(format!(
                "tenant-tier plugin may not hold admin-scope permission `{permission}`"
            ));
        }

        if ctx.host.permissions.contains(permission) {
            Ok(())
        } else {
            Err(format!("permission denied: `{permission}`"))
        }
    }
}

/// Convenience alias matching the `dioxus-extism` type signature.
pub type PermissionCapabilityCheckFn = CapabilityCheckFn<ZeitrakHostCtx>;
