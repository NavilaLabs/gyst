//! WASM-backed aggregate wrapper and repository host for plugin aggregates.
//!
//! Provides two public types:
//!
//! - [`PluginAggregate`] — an [`eventually::aggregate::Aggregate`] implementation
//!   that delegates event-folding to the plugin's `<type>__apply` WASM export.
//! - [`PluginAggregateHost`] — wraps `eventually-any`'s snapshot Repository and
//!   orchestrates load / command / save for a single plugin aggregate type.
//!
//! ## Stream naming (§9.1)
//!
//! Every instance is identified by `plugin.<plugin_id>.<aggregate_type>.<uuid>`.
//! This stream ID is stored in `PluginAggregate::stream_id` and used as the
//! `aggregate_id` primary key in the event-store tables.
//!
//! ## Sync / async contract
//!
//! `eventually::aggregate::Aggregate::apply` is synchronous, but WASM calls are
//! `async`.  `apply` uses [`tokio::task::block_in_place`] to call the plugin's
//! `__apply` export without blocking the async executor thread.
//!
//! This requires a **multi-threaded** Tokio runtime.  Integration tests that
//! exercise plugin aggregates must use `#[tokio::test(flavor = "multi_thread")]`.
//!
//! ## Runtime registry
//!
//! Because `eventually-any` deserialises the aggregate state via `serde` (which
//! skips the `#[serde(skip)]` runtime field), `apply` falls back to a
//! process-global runtime registry keyed by `plugin_id`.
//! `PluginAggregateHost::new` registers the runtime at construction time so that
//! every subsequent `apply` call can look it up even after a DB round-trip.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};

use async_trait::async_trait;
use dioxus_extism_host::PluginRuntime;
use dioxus_extism_protocol::{PluginId, SessionCtx};
use eventually::aggregate::{self, Aggregate, Root};
use eventually::aggregate::repository::{GetError, Getter, SaveError, Saver};
use eventually::message::Message as EMessage;
use eventually::serde::Json;
use eventually_any::snapshot::Repository;
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

// ── Global runtime registry ───────────────────────────────────────────────────

/// Process-global map from `plugin_id` → `PluginRuntime`.
///
/// Populated by [`PluginAggregateHost::new`] so that `PluginAggregate::apply`
/// can look up the runtime after the aggregate is deserialised from the DB
/// (at which point the `#[serde(skip)]` field is `None`).
static PLUGIN_RUNTIMES: OnceLock<Mutex<HashMap<String, Arc<PluginRuntime<ZeitrakHostCtx>>>>> =
    OnceLock::new();

/// Register a runtime for `plugin_id`.  Called at [`PluginAggregateHost`] construction.
fn register_plugin_runtime(plugin_id: &str, runtime: Arc<PluginRuntime<ZeitrakHostCtx>>) {
    let map = PLUGIN_RUNTIMES.get_or_init(|| Mutex::new(HashMap::new()));
    map.lock()
        .expect("plugin runtime registry poisoned")
        .insert(plugin_id.to_string(), runtime);
}

/// Look up the runtime for `plugin_id`, returning `None` if not registered.
fn lookup_plugin_runtime(plugin_id: &str) -> Option<Arc<PluginRuntime<ZeitrakHostCtx>>> {
    PLUGIN_RUNTIMES.get()?.lock().ok()?.get(plugin_id).cloned()
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
    /// Returns an interned `&'static str` for use in the event-store schema.
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
    /// No runtime available — register via [`PluginAggregateHost::new`] first.
    #[error("plugin aggregate has no runtime — register with PluginAggregateHost first")]
    NoRuntime,
    /// The `<type>__apply` WASM export returned an error.
    #[error("plugin WASM apply failed: {0}")]
    WasmCallFailed(String),
}

// ── PluginCommandError ────────────────────────────────────────────────────────

/// Errors produced by [`PluginAggregateHost::execute_command`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PluginCommandError {
    /// Failed to load the aggregate from the event store.
    #[error("aggregate load failed: {0}")]
    LoadFailed(String),
    /// The WASM call to `__handle_command` or `__initial_state` failed.
    #[error("plugin WASM call failed: {0}")]
    WasmCallFailed(String),
    /// The plugin rejected the command with a domain error.
    #[error("domain error: {0}")]
    DomainError(String),
    /// Applying an event produced by the command failed.
    #[error("event apply failed: {0}")]
    ApplyFailed(String),
    /// Persisting the aggregate state failed.
    #[error("aggregate save failed: {0}")]
    SaveFailed(String),
}

// ── NoOpAuthorizationRepository ───────────────────────────────────────────────

/// A zero-capability [`AuthorizationRepository`] for system-level WASM calls.
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

/// Builds a system-level [`ZeitrakHostCtx`] for use in `__apply` WASM calls.
/// Builds a system-level [`ZeitrakHostCtx`] for use in WASM calls that have no
/// user context (aggregate apply, projection handlers).
pub(crate) fn system_host_ctx() -> ZeitrakHostCtx {
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
/// The `aggregate_id()` returns `stream_id` — the full
/// `plugin.<plugin_id>.<aggregate_type>.<uuid>` prefix (§9.1) — so that all
/// plugin instances share the generic `type_name()` (`"plugin_aggregate"`) while
/// remaining uniquely addressable in the event-store tables.
#[derive(Clone, Serialize, Deserialize)]
pub struct PluginAggregate {
    /// Full event-stream ID: `plugin.<plugin_id>.<aggregate_type>.<uuid>`.
    pub stream_id: String,
    /// UUID portion of the stream ID — the value exposed in the HTTP API.
    pub uuid: String,
    /// Plugin that owns this aggregate type.
    pub plugin_id: String,
    /// Aggregate type name (e.g. `"leave_request"`).
    pub aggregate_type: String,
    /// Current state, serialised as an opaque JSON value.
    pub state: serde_json::Value,
    /// Runtime reference used in [`Aggregate::apply`].
    ///
    /// Not serialised — restored via the global runtime registry or
    /// [`PluginAggregate::with_runtime`] after DB deserialisation.
    #[serde(skip)]
    pub runtime: Option<Arc<PluginRuntime<ZeitrakHostCtx>>>,
}

impl std::fmt::Debug for PluginAggregate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginAggregate")
            .field("stream_id", &self.stream_id)
            .field("uuid", &self.uuid)
            .field("plugin_id", &self.plugin_id)
            .field("aggregate_type", &self.aggregate_type)
            .field("has_runtime", &self.runtime.is_some())
            .finish_non_exhaustive()
    }
}

impl PluginAggregate {
    /// Compute the full stream ID from its parts.
    #[must_use]
    pub fn make_stream_id(plugin_id: &str, aggregate_type: &str, uuid: &str) -> String {
        format!("plugin.{plugin_id}.{aggregate_type}.{uuid}")
    }

    /// Create a new, uninitialised aggregate instance.
    ///
    /// `state` starts as `Value::Null`; callers should populate it by
    /// calling the plugin's `__initial_state` export before recording events.
    #[must_use]
    pub fn new(
        uuid: impl Into<String>,
        plugin_id: impl Into<String>,
        aggregate_type: impl Into<String>,
        runtime: Arc<PluginRuntime<ZeitrakHostCtx>>,
    ) -> Self {
        let uuid = uuid.into();
        let plugin_id = plugin_id.into();
        let aggregate_type = aggregate_type.into();
        let stream_id = Self::make_stream_id(&plugin_id, &aggregate_type, &uuid);
        Self {
            stream_id,
            uuid,
            plugin_id,
            aggregate_type,
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
}

impl Aggregate for PluginAggregate {
    type Id = String;
    type Event = PluginEvent;
    type Error = PluginAggregateError;

    /// All plugin aggregates share a single type name.
    ///
    /// The full `plugin.<id>.<type>.<uuid>` stream ID encodes the actual type
    /// information so the `aggregates` table schema remains generic.
    fn type_name() -> &'static str {
        "plugin_aggregate"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.stream_id
    }

    /// Apply a domain event by delegating to the plugin's `<type>__apply` export.
    ///
    /// Uses [`tokio::task::block_in_place`] to call WASM synchronously; requires a
    /// **multi-threaded** Tokio runtime (panics in `current_thread` tests).
    ///
    /// The runtime is resolved in order: `self.runtime` → global registry →
    /// `Err(NoRuntime)`.  The found runtime is cached back on `self` to avoid
    /// repeated registry lookups.
    ///
    /// `apply(None, event)` is unsupported and returns [`PluginAggregateError::NoState`].
    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        let Some(mut current) = state else {
            return Err(PluginAggregateError::NoState);
        };

        let runtime = current
            .runtime
            .clone()
            .or_else(|| lookup_plugin_runtime(&current.plugin_id))
            .ok_or(PluginAggregateError::NoRuntime)?;

        // Cache the runtime so subsequent apply calls skip the registry lookup.
        current.runtime = Some(Arc::clone(&runtime));

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

// ── PluginAggregateHost ────────────────────────────────────────────────────────

/// Repository host for a single plugin aggregate type.
///
/// Wraps `eventually-any`'s snapshot [`Repository`] and provides the
/// [`execute_command`] method that orchestrates load → command dispatch →
/// event application → save.
///
/// Construct via [`PluginAggregateHost::new`], which also registers the
/// runtime in the process-global registry so `PluginAggregate::apply`
/// works correctly after DB deserialisation.
pub struct PluginAggregateHost {
    store: Repository<PluginAggregate, Json<PluginAggregate>, Json<PluginEvent>>,
    runtime: Arc<PluginRuntime<ZeitrakHostCtx>>,
    plugin_id: String,
    aggregate_type: String,
}

impl std::fmt::Debug for PluginAggregateHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginAggregateHost")
            .field("plugin_id", &self.plugin_id)
            .field("aggregate_type", &self.aggregate_type)
            .finish_non_exhaustive()
    }
}

impl PluginAggregateHost {
    /// Build a new host, running any pending event-store migrations.
    ///
    /// `pool` must be an `AnyPool` connected to the same database as the
    /// event-store (tenant pool for tenant-scoped aggregates).
    /// `snapshot_every` is taken from the `AggregateDecl::snapshot_every`
    /// manifest field (or a default of `50`).
    ///
    /// # Errors
    ///
    /// Returns an error if migrations fail.
    pub async fn new(
        pool: sqlx::AnyPool,
        runtime: Arc<PluginRuntime<ZeitrakHostCtx>>,
        plugin_id: impl Into<String>,
        aggregate_type: impl Into<String>,
        snapshot_every: usize,
    ) -> Result<Self, sqlx::migrate::MigrateError> {
        let plugin_id = plugin_id.into();

        // Register so apply() can find the runtime after serde round-trips.
        register_plugin_runtime(&plugin_id, Arc::clone(&runtime));

        let store = Repository::<PluginAggregate, _, _>::new(
            pool,
            Json::default(),
            Json::default(),
        )
        .await?
        .with_snapshot_every(snapshot_every);

        Ok(Self {
            store,
            runtime,
            plugin_id,
            aggregate_type: aggregate_type.into(),
        })
    }

    /// Returns the full stream ID for an aggregate instance UUID.
    #[must_use]
    pub fn stream_id(&self, uuid: &str) -> String {
        PluginAggregate::make_stream_id(&self.plugin_id, &self.aggregate_type, uuid)
    }

    /// Load the current aggregate root for the given UUID.
    ///
    /// Returns `None` when the aggregate does not yet exist.
    ///
    /// # Errors
    ///
    /// Returns `GetError` on storage failures.
    pub async fn load(
        &self,
        uuid: &str,
    ) -> Result<Option<Root<PluginAggregate>>, GetError> {
        match self.store.get(&self.stream_id(uuid)).await {
            Ok(root) => Ok(Some(root)),
            Err(GetError::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Execute a command on a plugin aggregate and persist the resulting events.
    ///
    /// 1. Loads the current state (or creates a fresh one via `__initial_state`).
    /// 2. Calls `<type>__handle_command(state, command)`.
    /// 3. Applies each returned event via `<type>__apply` (uses `block_in_place`).
    /// 4. Persists the aggregate root.
    ///
    /// Returns the list of events that were persisted.
    ///
    /// # Errors
    ///
    /// Returns [`PluginCommandError`] on any failure.
    pub async fn execute_command(
        &self,
        uuid: &str,
        command: serde_json::Value,
        session: &SessionCtx,
        host_ctx: &ZeitrakHostCtx,
    ) -> Result<Vec<PluginEvent>, PluginCommandError> {
        let sid = self.stream_id(uuid);
        let plugin_id = PluginId(self.plugin_id.clone());

        // Step 1: load or initialise.
        let mut root = match self.store.get(&sid).await {
            Ok(root) => root,
            Err(GetError::NotFound) => {
                let initial_state = self
                    .runtime
                    .call_plugin::<_, serde_json::Value>(
                        &plugin_id,
                        &format!("{}__initial_state", self.aggregate_type),
                        &(),
                        session,
                        host_ctx,
                    )
                    .await
                    .map_err(|e| PluginCommandError::WasmCallFailed(e.to_string()))?;

                let agg = PluginAggregate {
                    stream_id: sid.clone(),
                    uuid: uuid.to_string(),
                    plugin_id: self.plugin_id.clone(),
                    aggregate_type: self.aggregate_type.clone(),
                    state: initial_state,
                    runtime: Some(Arc::clone(&self.runtime)),
                };
                aggregate::Root::rehydrate_from_state(0, agg)
            }
            Err(e) => {
                return Err(PluginCommandError::LoadFailed(e.to_string()));
            }
        };

        // Step 2: handle command.
        let current_state = root.state.clone();
        let output = self
            .runtime
            .call_plugin::<_, HandleCommandOutput>(
                &plugin_id,
                &format!("{}__handle_command", self.aggregate_type),
                &(current_state, command),
                session,
                host_ctx,
            )
            .await
            .map_err(|e| PluginCommandError::WasmCallFailed(e.to_string()))?;

        // Step 3: handle output.
        let events = match output {
            HandleCommandOutput::Events(evts) => evts,
            HandleCommandOutput::Error(msg) => return Err(PluginCommandError::DomainError(msg)),
        };

        if events.is_empty() {
            return Ok(events);
        }

        // Step 4: apply events (each apply() call goes through __apply via block_in_place).
        for event in &events {
            root.record_that(eventually::event::Envelope::from(event.clone()))
                .map_err(|e| PluginCommandError::ApplyFailed(e.to_string()))?;
        }

        // Step 5: persist.
        self.store
            .save(&mut root)
            .await
            .map_err(|e: SaveError| PluginCommandError::SaveFailed(e.to_string()))?;

        Ok(events)
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
    fn make_stream_id_format() {
        assert_eq!(
            PluginAggregate::make_stream_id("my-org/my-plugin", "leave_request", "uuid-123"),
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
            stream_id: "plugin.p.t.x".to_string(),
            uuid: "x".to_string(),
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
