//! Typed structs for zeitrak-specific manifest extension values.
//!
//! Each struct corresponds to one `[extensions."zeitrak.*"]` block in a
//! plugin's `plugin.toml` and is deserialized from the opaque `serde_json::Value`
//! that `dioxus-extism` hands to the registered [`ManifestExtensionHandler`]s.

use serde::Deserialize;

// ── zeitrak.aggregates ───────────────────────────────────────────────────────

/// A command declared inside `[[extensions."zeitrak.aggregates"]]`.
#[derive(Debug, Clone, Deserialize)]
pub struct CommandDecl {
    /// Command name, e.g. `"Submit"`.
    pub name: String,
    /// Permission required to invoke the command, e.g. `"leave.submit"`.
    pub permission: String,
}

/// One aggregate declaration in `[[extensions."zeitrak.aggregates"]]`.
///
/// ```toml
/// [[extensions."zeitrak.aggregates"]]
/// name = "leave_request"
/// events = ["Submitted", "Approved", "Rejected"]
/// snapshot_every = 50
/// commands = [
///   { name = "Submit",  permission = "leave.submit" },
///   { name = "Approve", permission = "leave.approve" },
/// ]
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct AggregateDecl {
    /// Globally unique aggregate type name.
    pub name: String,
    /// Event variant names produced by this aggregate.
    #[serde(default)]
    pub events: Vec<String>,
    /// Snapshot every N events. Must be positive if set.
    pub snapshot_every: Option<u32>,
    /// Commands the aggregate handles.
    #[serde(default)]
    pub commands: Vec<CommandDecl>,
}

// ── zeitrak.projections ───────────────────────────────────────────────────────

/// One projection declaration in `[[extensions."zeitrak.projections"]]`.
///
/// ```toml
/// [[extensions."zeitrak.projections"]]
/// name = "pending_leaves"
/// table = "pending_leaves"       # becomes plugin_<id>__pending_leaves
/// events = ["Submitted", "Approved", "Rejected"]
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectionDecl {
    /// Globally unique projection name.
    pub name: String,
    /// Table name suffix; the host prepends `plugin_<sanitized_id>__`.
    pub table: String,
    /// Event names this projection listens to.
    #[serde(default)]
    pub events: Vec<String>,
}

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
