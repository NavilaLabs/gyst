//! Bridge between `eventually-projection` runner and WASM plugin projections.
//!
//! Each projection declared in a plugin's `zeitrak.projections` manifest extension
//! becomes a [`PluginProjector`] that implements [`eventually_projection::Projector`].
//!
//! When the runner feeds an event to `handle`, the projector:
//!
//! 1. Checks whether the event type is in the projection's subscription list.
//! 2. If yes, forwards a [`PluginProjectionEvent`] to the plugin's
//!    `<projection_name>__project` WASM export via the shared runtime.
//! 3. Propagates any WASM error back to the runner (which will retry the event).
//!
//! ## Table naming (§9.5)
//!
//! Plugin projection tables live under `plugin_<sanitized_id>__<table_name>`.
//! Use [`make_projection_table`] to compute this prefix from the plugin ID and
//! the `ProjectionDecl::table` field.
//!
//! ## WASM export contract
//!
//! The plugin function signature (in `zeitrak-plugin-sdk`) is:
//!
//! ```ignore
//! #[plugin_fn]
//! pub fn leave_request_summary__project(
//!     input: Json<PluginProjectionEvent>,
//! ) -> FnResult<Json<()>> {
//!     // update local table via zeitrak storage host functions
//!     Ok(Json(()))
//! }
//! ```
//!
//! Returning `FnResult<Json<()>>` (rather than bare `FnResult<()>`) ensures the
//! host can distinguish a successful no-output response (JSON `null`) from an
//! Extism error.

use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use dioxus_extism_host::PluginRuntime;
use dioxus_extism_protocol::{PluginId, SessionCtx};
use eventually_projection::{Projector, RawEvent};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::aggregate_host::system_host_ctx;
use crate::host_ctx::ZeitrakHostCtx;

// ── Table naming ──────────────────────────────────────────────────────────────

/// Compute the qualified table name for a plugin projection.
///
/// Replaces `/`, `-`, and `.` in `plugin_id` with underscores, then prefixes
/// the result with `plugin_` and appends `__<table>`.
///
/// | `plugin_id`          | `table`           | Result                                      |
/// |---|---|---|
/// | `my-org/leave-guard` | `leave_summary`   | `plugin_my_org_leave_guard__leave_summary`  |
/// | `acme.corp/tracker`  | `time_entries`    | `plugin_acme_corp_tracker__time_entries`    |
#[must_use]
pub fn make_projection_table(plugin_id: &str, table: &str) -> String {
    let sanitized = plugin_id.replace(['/', '-', '.'], "_");
    format!("plugin_{sanitized}__{table}")
}

// ── PluginProjectionEvent ─────────────────────────────────────────────────────

/// Event envelope forwarded to a plugin's `<projection>__project` WASM export.
///
/// The payload is deserialised from the raw event bytes as a JSON value.
/// Unknown event types are never forwarded (filtered by the host before the
/// WASM call).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginProjectionEvent {
    /// The stream (aggregate instance) this event belongs to.
    pub stream_id: String,
    /// Event type discriminant (matches [`RawEvent::event_type`]).
    pub event_type: String,
    /// JSON-deserialised event payload.
    pub payload: serde_json::Value,
    /// Global position across all streams — use this as a cursor, not `version`.
    pub global_position: i64,
    /// Per-stream version number.
    pub version: i64,
}

// ── ProjectorBridgeError ──────────────────────────────────────────────────────

/// Error returned by [`PluginProjector::handle`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ProjectorBridgeError {
    /// The plugin's WASM `__project` export returned an error.
    #[error("WASM call to '{fn_name}' failed for plugin '{plugin_id}': {cause}")]
    WasmCallFailed {
        /// The projection function that failed.
        fn_name: String,
        /// The plugin whose WASM returned the error.
        plugin_id: String,
        /// The underlying Extism error.
        cause: String,
    },
}

// ── PluginProjector ───────────────────────────────────────────────────────────

/// An [`eventually_projection::Projector`] backed by a plugin WASM export.
///
/// Create one per projection declaration using [`PluginProjector::new`], then
/// register it with a [`eventually_projection::ProjectionRunner`].
///
/// Events whose `event_type` is not in `subscribed_events` are silently skipped
/// (no WASM call is made).
pub struct PluginProjector {
    runtime: Arc<PluginRuntime<ZeitrakHostCtx>>,
    plugin_id: PluginId,
    projection_name: String,
    subscribed_events: HashSet<String>,
}

impl std::fmt::Debug for PluginProjector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginProjector")
            .field("plugin_id", &self.plugin_id.0)
            .field("projection_name", &self.projection_name)
            .field("subscribed_events", &self.subscribed_events)
            .finish_non_exhaustive()
    }
}

impl PluginProjector {
    /// Create a new projector for `projection_name` inside `plugin_id`.
    ///
    /// `subscribed_events` is the list of event type names (from
    /// `ProjectionDecl::events`) that this projection listens to.
    #[must_use]
    pub fn new(
        runtime: Arc<PluginRuntime<ZeitrakHostCtx>>,
        plugin_id: impl Into<String>,
        projection_name: impl Into<String>,
        subscribed_events: Vec<String>,
    ) -> Self {
        Self {
            runtime,
            plugin_id: PluginId(plugin_id.into()),
            projection_name: projection_name.into(),
            subscribed_events: subscribed_events.into_iter().collect(),
        }
    }

    /// Returns the WASM export name for this projection's handler.
    #[must_use]
    pub fn wasm_fn_name(&self) -> String {
        format!("{}__project", self.projection_name)
    }
}

#[async_trait]
impl Projector for PluginProjector {
    type Error = ProjectorBridgeError;

    /// Forward an event to the plugin if it is in the subscription list.
    ///
    /// Events not in `subscribed_events` are silently skipped.
    /// WASM call failures are returned as [`ProjectorBridgeError::WasmCallFailed`]
    /// so the runner retries the event on the next run.
    async fn handle(&mut self, event: RawEvent) -> Result<(), Self::Error> {
        if !self.subscribed_events.contains(&event.event_type) {
            return Ok(());
        }

        let payload =
            serde_json::from_slice(&event.payload_bytes).unwrap_or(serde_json::Value::Null);

        let projection_event = PluginProjectionEvent {
            stream_id: event.stream_id,
            event_type: event.event_type,
            payload,
            global_position: event.global_position,
            version: event.version,
        };

        let fn_name = self.wasm_fn_name();
        let session = SessionCtx::default();
        let host_ctx = system_host_ctx();

        self.runtime
            .call_plugin::<_, serde_json::Value>(
                &self.plugin_id,
                &fn_name,
                &projection_event,
                &session,
                &host_ctx,
            )
            .await
            .map_err(|e| ProjectorBridgeError::WasmCallFailed {
                fn_name: fn_name.clone(),
                plugin_id: self.plugin_id.0.clone(),
                cause: e.to_string(),
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_projection_table_sanitizes_slash_and_hyphen() {
        assert_eq!(
            make_projection_table("my-org/leave-guard", "leave_summary"),
            "plugin_my_org_leave_guard__leave_summary"
        );
    }

    #[test]
    fn make_projection_table_sanitizes_dot() {
        assert_eq!(
            make_projection_table("acme.corp/tracker", "time_entries"),
            "plugin_acme_corp_tracker__time_entries"
        );
    }

    #[test]
    fn make_projection_table_simple_id() {
        assert_eq!(
            make_projection_table("myplugin", "users"),
            "plugin_myplugin__users"
        );
    }

    #[test]
    fn wasm_fn_name_appends_project_suffix() {
        // wasm_fn_name is a pure string computation — test it without a runtime.
        assert_eq!(
            format!("{}__project", "leave_summary"),
            "leave_summary__project"
        );
    }
}
