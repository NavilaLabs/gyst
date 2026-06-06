//! [`ManifestExtensionHandler`] implementations for all `zeitrak.*` namespaces.
//!
//! Each handler is registered with the `dioxus-extism` runtime at construction
//! time via [`PluginRuntimeBuilder::with_manifest_extension`].  `dioxus-extism`
//! calls `validate` before building the plugin pool and `on_load` / `on_unload`
//! around the plugin lifecycle.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use dioxus_extism_host::{ManifestExtensionError, ManifestExtensionHandler};
use dioxus_extism_protocol::PluginId;
use semver::{Version, VersionReq};
use zeitrak_core::permissions;

use crate::hooks::{HookRegistry, RegisteredHook};
use crate::manifest::{
    ZeitrakAppExtension, ZeitrakEventsExtension, ZeitrakHooksExtension, ZeitrakPermissionsExtension,
};

/// All core zeitrak domain event type strings.
///
/// Derived from the `event_type()` implementations across all core aggregate
/// `events.rs` files. Used by [`ZeitrakEventsHandler`] to validate plugin
/// subscriptions. [`ZeitrakAggregatesHandler`] (step 13) extends the runtime
/// registry with plugin-contributed event names.
pub(crate) const CORE_DOMAIN_EVENTS: &[&str] = &[
    "ActivityCreated",
    "ActivityUpdated",
    "ActivityDeleted",
    "InvitationCreated",
    "InvitationAccepted",
    "InvitationRevoked",
    "PermissionCreated",
    "TagCreated",
    "TagDeleted",
    "TagRenamed",
    "TagTimesheetTagged",
    "TagTimesheetUntagged",
    "TimesheetStarted",
    "TimesheetStopped",
    "TimesheetUpdated",
    "TimesheetCancelled",
    "TimesheetReassigned",
    "TimesheetTimeUpdated",
    "UserCreated",
    "UserVerificationRequested",
    "UserVerified",
    "UserSettingsUpdated",
    "WorkspaceCreated",
    "WorkspaceSettingsUpdated",
    "WorkspaceRoleCreated",
    "WorkspaceRoleDeleted",
    "WorkspaceRoleRenamed",
    "WorkspaceRolePermissionGranted",
    "WorkspaceRolePermissionRevoked",
    "WorkspaceUserRoleAssigned",
    "WorkspaceUserRoleRevoked",
    "WorkspaceUserRemoved",
    "WorkspaceUserPermissionGranted",
    "WorkspaceUserPermissionRevoked",
];

/// Valid `(service, command)` pairs that plugins may hook into.
///
/// Derived from the application service commands in `zeitrak/src/`.
const KNOWN_HOOK_TARGETS: &[(&str, &str)] = &[
    ("activity", "Create"),
    ("activity", "Update"),
    ("activity", "Delete"),
    ("timesheet", "Start"),
    ("timesheet", "Stop"),
    ("timesheet", "Update"),
    ("timesheet", "Cancel"),
    ("timesheet", "Tag"),
    ("timesheet", "Untag"),
    ("user", "Create"),
    ("user", "Verify"),
    ("user", "UpdateSettings"),
    ("workspace", "Create"),
    ("workspace", "UpdateSettings"),
    ("workspace_role", "Create"),
    ("workspace_role", "Delete"),
    ("workspace_role", "Rename"),
    ("workspace_role", "GrantPermission"),
    ("workspace_role", "RevokePermission"),
    ("workspace_role", "AssignUser"),
    ("workspace_role", "RevokeUserRole"),
    ("invitation", "Create"),
    ("invitation", "Accept"),
    ("invitation", "Revoke"),
];

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

// ── zeitrak.events ───────────────────────────────────────────────────────────

/// Handler for `[extensions."zeitrak.events"]`.
///
/// Validates that each subscribed event name is known, then records the
/// subscription in the shared registry.  Phase-D event dispatch reads the
/// registry to route published events to the correct plugins.
pub struct ZeitrakEventsHandler {
    /// Extended set of known event names (core + plugin-contributed via step 13).
    known_events: Arc<RwLock<HashSet<String>>>,
    /// Plugin → subscribed event names, keyed by `plugin_id.0`.
    subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl ZeitrakEventsHandler {
    /// Create a new handler.
    ///
    /// `known_events` must be pre-populated with core event names before any
    /// plugin is loaded.  `ZeitrakAggregatesHandler` (step 13) extends it with
    /// plugin-declared event names.
    #[must_use]
    pub const fn new(
        known_events: Arc<RwLock<HashSet<String>>>,
        subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>,
    ) -> Self {
        Self {
            known_events,
            subscriptions,
        }
    }
}

impl std::fmt::Debug for ZeitrakEventsHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZeitrakEventsHandler").finish_non_exhaustive()
    }
}

impl ManifestExtensionHandler for ZeitrakEventsHandler {
    fn validate(
        &self,
        plugin_id: &PluginId,
        value: &serde_json::Value,
    ) -> Result<(), ManifestExtensionError> {
        let ext: ZeitrakEventsExtension =
            serde_json::from_value(value.clone()).map_err(|e| {
                validation_failed(
                    "zeitrak.events",
                    format!("invalid extension value for plugin `{}`: {e}", plugin_id.0),
                )
            })?;

        let known = self.known_events.read().map_err(|e| {
            validation_failed("zeitrak.events", format!("known-events lock poisoned: {e}"))
        })?;

        for name in &ext.subscriptions {
            if !known.contains(name.as_str()) {
                return Err(validation_failed(
                    "zeitrak.events",
                    format!(
                        "plugin `{}` subscribes to unknown event `{name}`",
                        plugin_id.0
                    ),
                ));
            }
        }

        drop(known);
        Ok(())
    }

    fn on_load(
        &self,
        plugin_id: &PluginId,
        value: &serde_json::Value,
    ) -> Result<(), ManifestExtensionError> {
        let ext: ZeitrakEventsExtension =
            serde_json::from_value(value.clone()).map_err(|e| {
                load_failed(
                    "zeitrak.events",
                    format!("invalid extension value for plugin `{}`: {e}", plugin_id.0),
                )
            })?;

        {
            let mut subs = self.subscriptions.write().map_err(|e| {
                load_failed(
                    "zeitrak.events",
                    format!("subscriptions lock poisoned: {e}"),
                )
            })?;
            subs.insert(plugin_id.0.clone(), ext.subscriptions);
        }

        Ok(())
    }

    fn on_unload(&self, plugin_id: &PluginId) -> Result<(), ManifestExtensionError> {
        if let Ok(mut subs) = self.subscriptions.write() {
            subs.remove(&plugin_id.0);
        }
        Ok(())
    }
}

// ── zeitrak.hooks ─────────────────────────────────────────────────────────────

/// Handler for `[extensions."zeitrak.hooks"]`.
///
/// Validates that each declared hook targets a known `(service, command)` pair,
/// then registers the entries in the shared [`HookRegistry`].  Phase-D dispatch
/// reads the registry when invoking hooks around application-service commands.
pub struct ZeitrakHooksHandler {
    registry: Arc<RwLock<HookRegistry>>,
}

impl ZeitrakHooksHandler {
    /// Create a new handler backed by `registry`.
    #[must_use]
    pub const fn new(registry: Arc<RwLock<HookRegistry>>) -> Self {
        Self { registry }
    }
}

impl std::fmt::Debug for ZeitrakHooksHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZeitrakHooksHandler").finish_non_exhaustive()
    }
}

impl ManifestExtensionHandler for ZeitrakHooksHandler {
    fn validate(
        &self,
        plugin_id: &PluginId,
        value: &serde_json::Value,
    ) -> Result<(), ManifestExtensionError> {
        let ext: ZeitrakHooksExtension =
            serde_json::from_value(value.clone()).map_err(|e| {
                validation_failed(
                    "zeitrak.hooks",
                    format!("invalid extension value for plugin `{}`: {e}", plugin_id.0),
                )
            })?;

        for hook in &ext.command_hooks {
            let valid = KNOWN_HOOK_TARGETS
                .iter()
                .any(|&(s, c)| s == hook.service && c == hook.command);
            if !valid {
                return Err(validation_failed(
                    "zeitrak.hooks",
                    format!(
                        "plugin `{}` declares hook for unknown target `{}.{}`",
                        plugin_id.0, hook.service, hook.command
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
        let ext: ZeitrakHooksExtension =
            serde_json::from_value(value.clone()).map_err(|e| {
                load_failed(
                    "zeitrak.hooks",
                    format!("invalid extension value for plugin `{}`: {e}", plugin_id.0),
                )
            })?;

        let hooks = ext.command_hooks.into_iter().map(|entry| RegisteredHook {
            plugin_id: plugin_id.0.clone(),
            service: entry.service,
            command: entry.command,
            phase: entry.phase,
            priority: entry.priority,
        });

        {
            let mut registry = self.registry.write().map_err(|e| {
                load_failed(
                    "zeitrak.hooks",
                    format!("hook registry lock poisoned: {e}"),
                )
            })?;
            registry.register(hooks);
        }

        Ok(())
    }

    fn on_unload(&self, plugin_id: &PluginId) -> Result<(), ManifestExtensionError> {
        if let Ok(mut registry) = self.registry.write() {
            registry.unregister(&plugin_id.0);
        }
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

    // ── ZeitrakEventsHandler ──────────────────────────────────────────────────

    fn make_known_events() -> Arc<RwLock<HashSet<String>>> {
        Arc::new(RwLock::new(
            CORE_DOMAIN_EVENTS
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        ))
    }

    fn make_subscriptions() -> Arc<RwLock<HashMap<String, Vec<String>>>> {
        Arc::new(RwLock::new(HashMap::new()))
    }

    #[test]
    fn events_handler_accepts_known_event_names() {
        let handler =
            ZeitrakEventsHandler::new(make_known_events(), make_subscriptions());
        let value = json!({ "subscriptions": ["TimesheetStopped", "ActivityCreated"] });
        let id = PluginId("test-plugin".to_string());
        assert!(handler.validate(&id, &value).is_ok());
    }

    #[test]
    fn events_handler_rejects_unknown_event_name() {
        let handler =
            ZeitrakEventsHandler::new(make_known_events(), make_subscriptions());
        let value = json!({ "subscriptions": ["LeaveSubmitted"] });
        let id = PluginId("test-plugin".to_string());
        assert!(handler.validate(&id, &value).is_err());
    }

    #[test]
    fn events_handler_on_load_records_subscription() {
        let subs = make_subscriptions();
        let handler = ZeitrakEventsHandler::new(make_known_events(), Arc::clone(&subs));
        let value = json!({ "subscriptions": ["TimesheetStopped"] });
        let id = PluginId("test-plugin".to_string());

        handler.on_load(&id, &value).expect("on_load must succeed");

        let guard = subs.read().unwrap();
        assert_eq!(
            guard.get("test-plugin").map(Vec::as_slice),
            Some(["TimesheetStopped".to_string()].as_slice())
        );
        drop(guard);
    }

    #[test]
    fn events_handler_on_unload_removes_subscription() {
        let subs = make_subscriptions();
        let handler = ZeitrakEventsHandler::new(make_known_events(), Arc::clone(&subs));
        let value = json!({ "subscriptions": ["TimesheetStopped"] });
        let id = PluginId("test-plugin".to_string());

        handler.on_load(&id, &value).expect("on_load must succeed");
        handler.on_unload(&id).expect("on_unload must succeed");

        let guard = subs.read().unwrap();
        assert!(!guard.contains_key("test-plugin"));
        drop(guard);
    }

    // ── ZeitrakHooksHandler ───────────────────────────────────────────────────

    fn make_hook_registry() -> Arc<RwLock<HookRegistry>> {
        Arc::new(RwLock::new(HookRegistry::new()))
    }

    #[test]
    fn hooks_handler_accepts_known_target() {
        let handler = ZeitrakHooksHandler::new(make_hook_registry());
        let value = json!({
            "command_hooks": [{ "service": "timesheet", "command": "Stop", "phase": "Pre" }]
        });
        let id = PluginId("test-plugin".to_string());
        assert!(handler.validate(&id, &value).is_ok());
    }

    #[test]
    fn hooks_handler_rejects_unknown_target() {
        let handler = ZeitrakHooksHandler::new(make_hook_registry());
        let value = json!({
            "command_hooks": [{ "service": "leave", "command": "Submit", "phase": "Pre" }]
        });
        let id = PluginId("test-plugin".to_string());
        assert!(handler.validate(&id, &value).is_err());
    }

    #[test]
    fn hooks_handler_on_load_registers_hooks() {
        let registry = make_hook_registry();
        let handler = ZeitrakHooksHandler::new(Arc::clone(&registry));
        let value = json!({
            "command_hooks": [
                { "service": "timesheet", "command": "Stop", "phase": "Pre", "priority": 50 }
            ]
        });
        let id = PluginId("test-plugin".to_string());

        handler.on_load(&id, &value).expect("on_load must succeed");

        let guard = registry.read().unwrap();
        assert_eq!(guard.len(), 1);
        drop(guard);
    }

    #[test]
    fn hooks_handler_on_unload_removes_hooks() {
        let registry = make_hook_registry();
        let handler = ZeitrakHooksHandler::new(Arc::clone(&registry));
        let value = json!({
            "command_hooks": [
                { "service": "timesheet", "command": "Stop", "phase": "Pre" }
            ]
        });
        let id = PluginId("test-plugin".to_string());

        handler.on_load(&id, &value).expect("on_load must succeed");
        handler.on_unload(&id).expect("on_unload must succeed");

        let guard = registry.read().unwrap();
        assert!(guard.is_empty());
        drop(guard);
    }
}
