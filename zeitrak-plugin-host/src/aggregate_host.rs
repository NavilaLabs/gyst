//! WASM-backed aggregate wrapper for plugin-authored event-sourced aggregates.
//!
//! Each plugin aggregate declared in `zeitrak.aggregates` gets a [`PluginAggregate`]
//! runtime wrapper that implements [`eventually::aggregate::Aggregate`] and delegates
//! to three WASM exports:
//!
//! | Export | Signature |
//! |---|---|
//! | `<type>__apply` | `Json<(state, event)> -> Json<state>` |
//! | `<type>__handle_command` | `Json<(state, command)> -> Json<HandleCommandOutput>` |
//! | `<type>__initial_state` | `Json<()> -> Json<state>` |
//!
//! ## Sync / async contract
//!
//! `eventually::aggregate::Aggregate::apply` is synchronous, but WASM calls via
//! `PluginRuntime::call_plugin` are `async` (they use `spawn_blocking` internally).
//! `apply` uses [`tokio::task::block_in_place`] to perform the blocking WASM call
//! without blocking the async executor thread.
//!
//! This requires a **multi-threaded** Tokio runtime. Integration tests that exercise
//! plugin aggregates must use `#[tokio::test(flavor = "multi_thread")]`.
//!
//! ## `None`-state guard
//!
//! `apply(None, event)` is never called in the normal command-handling path (Phase E
//! uses [`PluginAggregateHost`] to pre-populate the runtime before recording events).
//! If it is called, `apply` returns [`PluginAggregateError::NoState`].

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};

use async_trait::async_trait;
use dioxus_extism_host::PluginRuntime;
use dioxus_extism_protocol::{PluginId, SessionCtx};
use eventually::aggregate::Aggregate;
use eventually::message::Message as EMessage;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeitrak_core::shared::clock::SystemClock;
use zeitrak_infrastructure::authorization::{AuthorizationError, AuthorizationRepository};

use crate::host_ctx::{PermissionSet, ZeitrakHostCtx};
use crate::trust::ZeitrakTrustTier;

// ── String interning ──────────────────────────────────────────────────────────

/// Returns a `&'static str` for the given event name, interning it on first use.
///
/// Uses `Box::leak` to produce the static reference; total leaked memory is
/// bounded by the number of unique plugin event type names across all loaded
/// plugins (typically a few dozen strings).
fn intern_event_name(name: &str) -> &'static str {
    static INTERN: OnceLock<Mutex<HashMap<String, &'static str>>> = OnceLock::new();
    let map = INTERN.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map.lock().expect("event name intern table poisoned");
    if let Some(&s) = guard.get(name) {
        return s;
    }
    let leaked: &'static str = Box::leak(name.to_string().into_boxed_str());
    guard.insert(name.to_string(), leaked);
    leaked
}

// ── PluginEvent ────────────────────────────────────────────────────────────────

/// A domain event emitted or applied by a plugin aggregate.
///
/// `event_type` is the event variant name (e.g. `"LeaveRequestSubmitted"`).
/// `payload` is the JSON-encoded event data forwarded to and from the plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEvent {
    /// Event variant name (e.g. `"LeaveRequestSubmitted"`).
    pub event_type: String,
    /// JSON-encoded event payload.
    pub payload: serde_json::Value,
}

impl EMessage for PluginEvent {
    /// Returns a `&'static str` event name, interned on first use.
    fn name(&self) -> &'static str {
        intern_event_name(&self.event_type)
    }
}

// ── HandleCommandOutput ────────────────────────────────────────────────────────

/// Returned by a plugin's `<aggregate>__handle_command` WASM export.
///
/// On success the plugin returns the list of events to persist; on domain-rule
/// violation it returns a human-readable error message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum HandleCommandOutput {
    /// The command succeeded; these events should be persisted and applied.
    Events(Vec<PluginEvent>),
    /// The command was rejected by a domain invariant.
    Error(String),
}

// ── PluginAggregateError ──────────────────────────────────────────────────────

/// Errors produced by [`PluginAggregate::apply`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PluginAggregateError {
    /// `apply(None, event)` was called; callers must pre-populate the state.
    #[error("plugin aggregate apply called with no prior state")]
    NoState,
    /// The runtime reference was missing after deserialisation.
    #[error("plugin aggregate has no runtime — call with_runtime() after loading")]
    NoRuntime,
    /// The `<type>__apply` WASM export returned an error.
    #[error("plugin WASM apply failed: {0}")]
    WasmCallFailed(String),
}

// ── NoOpAuthorizationRepository ───────────────────────────────────────────────

/// A zero-capability [`AuthorizationRepository`] for system-level apply calls.
///
/// Plugin aggregate apply calls run without a user session; permission checks
/// inside `__apply` WASM exports are expected to be read-only or bypass
/// authorisation entirely.
struct NoOpAuthorizationRepository;

#[async_trait]
impl AuthorizationRepository for NoOpAuthorizationRepository {
    async fn is_admin(&self, _user_id: &str) -> Result<bool, AuthorizationError> {
        Ok(false)
    }

    async fn has_permission(
        &self,
        _user_id: &str,
        _workspace_id: &str,
        _permission: &str,
    ) -> Result<bool, AuthorizationError> {
        Ok(false)
    }

    async fn user_permissions(
        &self,
        _user_id: &str,
        _workspace_id: &str,
    ) -> Result<HashSet<String>, AuthorizationError> {
        Ok(HashSet::new())
    }
}

/// Builds a system-level [`ZeitrakHostCtx`] for use in plugin aggregate apply calls.
fn system_host_ctx() -> ZeitrakHostCtx {
    ZeitrakHostCtx {
        user_id: None,
        workspace_id: None,
        permissions: Arc::new(PermissionSet::new(HashSet::new())),
        trust_tier: ZeitrakTrustTier::Tenant,
        authz: Arc::new(NoOpAuthorizationRepository),
        clock: Arc::new(SystemClock),
    }
}

// ── PluginAggregate ────────────────────────────────────────────────────────────

/// WASM-backed aggregate state for a plugin-authored aggregate type.
///
/// Implements [`eventually::aggregate::Aggregate`] so the state can be stored
/// and loaded via `eventually-any`'s snapshot repository.
///
/// All plugin aggregates share the same `type_name()` (`"plugin_aggregate"`);
/// the stream ID carries the type information via the
/// `plugin.<plugin_id>.<aggregate_type>.<uuid>` prefix (§9.1).
#[derive(Clone, Serialize, Deserialize)]
pub struct PluginAggregate {
    /// Aggregate instance identifier (stream UUID suffix, not the full stream ID).
    pub id: String,
    /// Plugin that owns this aggregate type.
    pub plugin_id: String,
    /// Aggregate type name, e.g. `"leave_request"`.
    pub aggregate_type: String,
    /// Current state, serialised as an opaque JSON value.
    pub state: serde_json::Value,

    /// Runtime reference used in [`Aggregate::apply`].
    ///
    /// Not serialised — must be restored with [`PluginAggregate::with_runtime`]
    /// after loading from the DB.
    #[serde(skip)]
    pub runtime: Option<Arc<PluginRuntime<ZeitrakHostCtx>>>,
}

impl std::fmt::Debug for PluginAggregate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginAggregate")
            .field("id", &self.id)
            .field("plugin_id", &self.plugin_id)
            .field("aggregate_type", &self.aggregate_type)
            .field("has_runtime", &self.runtime.is_some())
            .finish_non_exhaustive()
    }
}

impl PluginAggregate {
    /// Create a new, uninitialised aggregate instance with the given metadata.
    ///
    /// `state` starts as `Value::Null`; callers should populate it by
    /// calling the plugin's `__initial_state` export before recording events.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        plugin_id: impl Into<String>,
        aggregate_type: impl Into<String>,
        runtime: Arc<PluginRuntime<ZeitrakHostCtx>>,
    ) -> Self {
        Self {
            id: id.into(),
            plugin_id: plugin_id.into(),
            aggregate_type: aggregate_type.into(),
            state: serde_json::Value::Null,
            runtime: Some(runtime),
        }
    }

    /// Restore the runtime reference after deserialisation from the DB.
    #[must_use]
    pub fn with_runtime(mut self, runtime: Arc<PluginRuntime<ZeitrakHostCtx>>) -> Self {
        self.runtime = Some(runtime);
        self
    }

    /// Returns the full event-stream ID for this aggregate instance.
    ///
    /// Format: `plugin.<plugin_id>.<aggregate_type>.<id>` (§9.1).
    #[must_use]
    pub fn stream_id(&self) -> String {
        format!("plugin.{}.{}.{}", self.plugin_id, self.aggregate_type, self.id)
    }
}

impl Aggregate for PluginAggregate {
    type Id = String;
    type Event = PluginEvent;
    type Error = PluginAggregateError;

    /// All plugin aggregates share a single `type_name`.
    ///
    /// The stream ID (`plugin.<plugin_id>.<type>.<uuid>`) encodes the actual
    /// aggregate type, keeping the `aggregates` table schema generic.
    fn type_name() -> &'static str {
        "plugin_aggregate"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    /// Apply a domain event by delegating to the plugin's `<type>__apply` export.
    ///
    /// Requires a **multi-threaded** Tokio runtime; panics inside a
    /// `current_thread` runtime (e.g. the default `#[tokio::test]`).
    ///
    /// `apply(None, event)` is not supported — callers must pre-populate the
    /// state before recording events. Returns [`PluginAggregateError::NoState`]
    /// if called with `None`.
    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        let Some(mut current) = state else {
            return Err(PluginAggregateError::NoState);
        };

        let Some(runtime) = current.runtime.clone() else {
            return Err(PluginAggregateError::NoRuntime);
        };

        let plugin_id = PluginId(current.plugin_id.clone());
        let fn_name = format!("{}__apply", current.aggregate_type);
        let input = (current.state.clone(), event.payload);
        let session = SessionCtx::default();
        let host_ctx = system_host_ctx();

        let new_state = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                runtime.call_plugin::<_, serde_json::Value>(
                    &plugin_id,
                    &fn_name,
                    &input,
                    &session,
                    &host_ctx,
                ),
            )
        })
        .map_err(|e| PluginAggregateError::WasmCallFailed(e.to_string()))?;

        current.state = new_state;
        Ok(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern_event_name_returns_same_pointer_for_same_string() {
        let a = intern_event_name("LeaveRequestSubmitted");
        let b = intern_event_name("LeaveRequestSubmitted");
        assert!(std::ptr::eq(a, b), "same string should yield same pointer");
    }

    #[test]
    fn intern_event_name_returns_different_pointers_for_different_strings() {
        let a = intern_event_name("EventA");
        let b = intern_event_name("EventB");
        assert!(!std::ptr::eq(a, b));
    }

    #[test]
    fn plugin_event_message_name_delegates_to_intern() {
        let ev = PluginEvent {
            event_type: "TestEvent".to_string(),
            payload: serde_json::Value::Null,
        };
        assert_eq!(ev.name(), "TestEvent");
    }

    #[test]
    fn plugin_aggregate_stream_id_format() {
        let agg = PluginAggregate {
            id: "uuid-123".to_string(),
            plugin_id: "my-org/my-plugin".to_string(),
            aggregate_type: "leave_request".to_string(),
            state: serde_json::Value::Null,
            runtime: None,
        };
        assert_eq!(
            agg.stream_id(),
            "plugin.my-org/my-plugin.leave_request.uuid-123"
        );
    }

    #[test]
    fn apply_none_state_returns_no_state_error() {
        let event = PluginEvent {
            event_type: "Created".to_string(),
            payload: serde_json::json!({ "name": "Test" }),
        };
        let result = PluginAggregate::apply(None, event);
        assert!(matches!(result, Err(PluginAggregateError::NoState)));
    }

    #[test]
    fn apply_without_runtime_returns_no_runtime_error() {
        let agg = PluginAggregate {
            id: "x".to_string(),
            plugin_id: "p".to_string(),
            aggregate_type: "t".to_string(),
            state: serde_json::Value::Null,
            runtime: None,
        };
        let event = PluginEvent {
            event_type: "Updated".to_string(),
            payload: serde_json::Value::Null,
        };
        let result = PluginAggregate::apply(Some(agg), event);
        assert!(matches!(result, Err(PluginAggregateError::NoRuntime)));
    }
}
