//! Typed structs for zeitrak-specific manifest extension values.
//!
//! Each struct corresponds to one `[extensions."zeitrak.*"]` block in a
//! plugin's `plugin.toml` and is deserialized from the opaque `serde_json::Value`
//! that `dioxus-extism` hands to the registered [`ManifestExtensionHandler`]s.

use serde::Deserialize;

// ── zeitrak.events ───────────────────────────────────────────────────────────

/// `[extensions."zeitrak.events"]`
///
/// ```toml
/// [extensions."zeitrak.events"]
/// subscriptions = ["TimesheetStopped", "ActivityCreated"]
/// ```
#[derive(Debug, Deserialize)]
pub struct ZeitrakEventsExtension {
    /// Domain event names the plugin subscribes to.
    ///
    /// Each name must match a known core or plugin-contributed event type.
    #[serde(default)]
    pub subscriptions: Vec<String>,
}

// ── zeitrak.hooks ─────────────────────────────────────────────────────────────

/// Hook phase — whether the hook runs before or after the command executes.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub enum HookPhase {
    /// Runs before the command; may cancel or replace the input.
    Pre,
    /// Runs after the command has completed; fire-and-forget.
    Post,
}

/// A single hook declaration inside `zeitrak.hooks.command_hooks`.
#[derive(Debug, Deserialize)]
pub struct CommandHookEntry {
    /// Service name, e.g. `"timesheet"`.
    pub service: String,
    /// Command name, e.g. `"Stop"`.
    pub command: String,
    /// Whether the hook runs before or after the command.
    pub phase: HookPhase,
    /// Dispatch priority. Lower values run first; defaults to 100.
    #[serde(default = "default_hook_priority")]
    pub priority: i32,
}

const fn default_hook_priority() -> i32 {
    100
}

/// `[extensions."zeitrak.hooks"]`
///
/// ```toml
/// [extensions."zeitrak.hooks"]
/// command_hooks = [
///   { service = "timesheet", command = "Stop", phase = "Pre", priority = 100 },
/// ]
/// ```
#[derive(Debug, Deserialize)]
pub struct ZeitrakHooksExtension {
    /// Command hooks declared by the plugin.
    #[serde(default)]
    pub command_hooks: Vec<CommandHookEntry>,
}

// ── zeitrak.app ───────────────────────────────────────────────────────────────

/// `[extensions."zeitrak.app"]`
///
/// ```toml
/// [extensions."zeitrak.app"]
/// min_version = "0.5"
/// ```
#[derive(Debug, Deserialize)]
pub struct ZeitrakAppExtension {
    /// Minimum zeitrak version required by the plugin (semver string).
    pub min_version: String,
}

// ── zeitrak.permissions ───────────────────────────────────────────────────────

/// `[extensions."zeitrak.permissions"]`
///
/// ```toml
/// [extensions."zeitrak.permissions"]
/// contributed = ["leave.submit", "leave.approve"]
/// ```
#[derive(Debug, Deserialize)]
pub struct ZeitrakPermissionsExtension {
    /// Permission names contributed by the plugin.
    ///
    /// These are added to the runtime permission registry and can be granted to
    /// users. Must not conflict with core permission names or use the `admin.`
    /// prefix.
    #[serde(default)]
    pub contributed: Vec<String>,
}
