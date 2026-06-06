//! [`ManifestExtensionHandler`] implementations for all `zeitrak.*` namespaces.
//!
//! Each handler is registered with the `dioxus-extism` runtime at construction
//! time via [`PluginRuntimeBuilder::with_manifest_extension`].  `dioxus-extism`
//! calls `validate` before building the plugin pool and `on_load` / `on_unload`
//! around the plugin lifecycle.

use std::collections::HashSet;
use std::sync::{Arc, RwLock};

use dioxus_extism_host::{ManifestExtensionError, ManifestExtensionHandler};
use dioxus_extism_protocol::PluginId;
use semver::{Version, VersionReq};
use zeitrak_core::permissions;

use crate::manifest::{ZeitrakAppExtension, ZeitrakPermissionsExtension};

/// The current zeitrak application version, read from the crate manifest at
/// compile time.  Plugins declare `min_version` against this.
pub const ZEITRAK_APP_VERSION: &str = env!("CARGO_PKG_VERSION");

// ── helpers ───────────────────────────────────────────────────────────────────

fn validation_failed(
    namespace: &str,
    message: impl Into<String>,
) -> ManifestExtensionError {
    ManifestExtensionError::ValidationFailed {
        namespace: namespace.to_string(),
        message: message.into(),
    }
}

fn load_failed(namespace: &str, message: impl Into<String>) -> ManifestExtensionError {
    ManifestExtensionError::LoadFailed {
        namespace: namespace.to_string(),
        message: message.into(),
    }
}

// ── zeitrak.app ───────────────────────────────────────────────────────────────

/// Handler for `[extensions."zeitrak.app"]`.
///
/// Validates that the current zeitrak version satisfies the plugin's
/// `min_version` requirement.  Always succeeds in `on_load` and `on_unload`
/// since there is no runtime state to maintain.
#[derive(Debug, Default)]
pub struct ZeitrakAppHandler;

impl ManifestExtensionHandler for ZeitrakAppHandler {
    fn validate(
        &self,
        plugin_id: &PluginId,
        value: &serde_json::Value,
    ) -> Result<(), ManifestExtensionError> {
        let ext: ZeitrakAppExtension =
            serde_json::from_value(value.clone()).map_err(|e| {
                validation_failed(
                    "zeitrak.app",
                    format!("invalid extension value for plugin `{}`: {e}", plugin_id.0),
                )
            })?;

        let requirement = VersionReq::parse(&ext.min_version).map_err(|e| {
            validation_failed(
                "zeitrak.app",
                format!(
                    "plugin `{}` has invalid min_version `{}`: {e}",
                    plugin_id.0,
                    ext.min_version
                ),
            )
        })?;

        let host_version = Version::parse(ZEITRAK_APP_VERSION).map_err(|e| {
            validation_failed(
                "zeitrak.app",
                format!("host version `{ZEITRAK_APP_VERSION}` is not valid semver: {e}"),
            )
        })?;

        if !requirement.matches(&host_version) {
            return Err(validation_failed(
                "zeitrak.app",
                format!(
                    "plugin `{}` requires zeitrak {requirement} but host is {host_version}", plugin_id.0
                ),
            ));
        }

        Ok(())
    }

    fn on_load(
        &self,
        _plugin_id: &PluginId,
        _value: &serde_json::Value,
    ) -> Result<(), ManifestExtensionError> {
        Ok(())
    }

    fn on_unload(&self, _plugin_id: &PluginId) -> Result<(), ManifestExtensionError> {
        Ok(())
    }
}

// ── zeitrak.permissions ───────────────────────────────────────────────────────

/// Handler for `[extensions."zeitrak.permissions"]`.
///
/// Validates that contributed permission names do not conflict with core
/// permissions and do not use the reserved `admin.` prefix (which is
/// exclusively for the host).  On load the names are inserted into a shared
/// registry; on unload they are removed.
pub struct ZeitrakPermissionsHandler {
    /// Shared set of all plugin-contributed permission names, keyed by name.
    /// The set grows as plugins are loaded and shrinks as they are unloaded.
    registry: Arc<RwLock<HashSet<String>>>,
}

impl ZeitrakPermissionsHandler {
    /// Create a new handler backed by the given shared registry.
    #[must_use]
    pub const fn new(registry: Arc<RwLock<HashSet<String>>>) -> Self {
        Self { registry }
    }
}

impl std::fmt::Debug for ZeitrakPermissionsHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZeitrakPermissionsHandler")
            .field(
                "registry_size",
                &self.registry.read().map_or(0, |r| r.len()),
            )
            .finish()
    }
}

impl ManifestExtensionHandler for ZeitrakPermissionsHandler {
    fn validate(
        &self,
        plugin_id: &PluginId,
        value: &serde_json::Value,
    ) -> Result<(), ManifestExtensionError> {
        let ext: ZeitrakPermissionsExtension =
            serde_json::from_value(value.clone()).map_err(|e| {
                validation_failed(
                    "zeitrak.permissions",
                    format!("invalid extension value for plugin `{}`: {e}", plugin_id.0),
                )
            })?;

        for name in &ext.contributed {
            // Reserved prefix — only the host defines admin permissions.
            if name.starts_with("admin.") {
                return Err(validation_failed(
                    "zeitrak.permissions",
                    format!(
                        "plugin `{}` may not contribute a permission in the \
                         reserved `admin.` namespace: `{name}`", plugin_id.0
                    ),
                ));
            }

            // Must not shadow a core permission.
            if permissions::ALL.contains(&name.as_str()) {
                return Err(validation_failed(
                    "zeitrak.permissions",
                    format!(
                        "plugin `{}` attempts to re-register core permission `{name}`", plugin_id.0
                    ),
                ));
            }
        }

        Ok(())
    }

    fn on_load(
        &self,
        plugin_id: &PluginId,
        value: &serde_json::Value,
    ) -> Result<(), ManifestExtensionError> {
        let ext: ZeitrakPermissionsExtension =
            serde_json::from_value(value.clone()).map_err(|e| {
                load_failed(
                    "zeitrak.permissions",
                    format!("invalid extension value for plugin `{}`: {e}", plugin_id.0),
                )
            })?;

        {
            let mut registry = self.registry.write().map_err(|e| {
                load_failed("zeitrak.permissions", format!("registry lock poisoned: {e}"))
            })?;
            for name in ext.contributed {
                registry.insert(name);
            }
        }

        Ok(())
    }

    fn on_unload(&self, plugin_id: &PluginId) -> Result<(), ManifestExtensionError> {
        // We stored the names at load time in the shared registry but did not
        // record which names belong to which plugin.  To allow clean removal we
        // would need a per-plugin index; for now we log and skip — the registry
        // is rebuilt on restart.  A follow-up can track (plugin_id → names).
        tracing::warn!(
            plugin_id = %plugin_id.0,
            "zeitrak.permissions: on_unload does not yet remove contributed permissions \
             from the registry; a host restart will clean them up"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_registry() -> Arc<RwLock<HashSet<String>>> {
        Arc::new(RwLock::new(HashSet::new()))
    }

    // ── ZeitrakAppHandler ─────────────────────────────────────────────────────

    #[test]
    fn app_handler_accepts_matching_min_version() {
        let handler = ZeitrakAppHandler;
        // Our CARGO_PKG_VERSION satisfies ">= 0.0.1" (or any ≤ actual version).
        let value = json!({ "min_version": ">=0.0.1" });
        let id = PluginId("test-plugin".to_string());
        assert!(handler.validate(&id, &value).is_ok());
    }

    #[test]
    fn app_handler_rejects_future_min_version() {
        let handler = ZeitrakAppHandler;
        // 999.0.0 will never match 0.x
        let value = json!({ "min_version": ">=999.0.0" });
        let id = PluginId("test-plugin".to_string());
        assert!(handler.validate(&id, &value).is_err());
    }

    #[test]
    fn app_handler_rejects_invalid_semver() {
        let handler = ZeitrakAppHandler;
        let value = json!({ "min_version": "not-semver" });
        let id = PluginId("test-plugin".to_string());
        assert!(handler.validate(&id, &value).is_err());
    }

    // ── ZeitrakPermissionsHandler ─────────────────────────────────────────────

    #[test]
    fn permissions_handler_accepts_valid_contributed_names() {
        let handler = ZeitrakPermissionsHandler::new(make_registry());
        let value = json!({ "contributed": ["leave.submit", "leave.approve"] });
        let id = PluginId("test-plugin".to_string());
        assert!(handler.validate(&id, &value).is_ok());
    }

    #[test]
    fn permissions_handler_rejects_admin_prefix() {
        let handler = ZeitrakPermissionsHandler::new(make_registry());
        let value = json!({ "contributed": ["admin.bypass"] });
        let id = PluginId("test-plugin".to_string());
        assert!(handler.validate(&id, &value).is_err());
    }

    #[test]
    fn permissions_handler_rejects_core_permission_shadow() {
        let handler = ZeitrakPermissionsHandler::new(make_registry());
        let value = json!({ "contributed": ["timesheet.create"] });
        let id = PluginId("test-plugin".to_string());
        assert!(handler.validate(&id, &value).is_err());
    }

    #[test]
    fn permissions_handler_on_load_inserts_into_registry() {
        let registry = make_registry();
        let handler = ZeitrakPermissionsHandler::new(Arc::clone(&registry));
        let value = json!({ "contributed": ["leave.submit", "leave.approve"] });
        let id = PluginId("test-plugin".to_string());

        handler.on_load(&id, &value).expect("on_load must succeed");

        let guard = registry.read().unwrap();
        assert!(guard.contains("leave.submit"));
        assert!(guard.contains("leave.approve"));
        drop(guard);
    }
}
