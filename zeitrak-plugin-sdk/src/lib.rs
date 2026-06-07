//! Plugin Development Kit for zeitrak.
//!
//! One-stop import for zeitrak plugin authors.  Provides zeitrak-specific
//! types, traits, and macros on top of `dioxus-extism-pdk`.
//!
//! # Quick start
//!
//! ```ignore
//! use zeitrak_plugin_sdk::*;
//!
//! struct LeaveGuardPlugin;
//!
//! impl DioxusPlugin for LeaveGuardPlugin {
//!     fn manifest() -> PluginManifest {
//!         PluginManifest {
//!             id: PluginId("my-org/leave-guard".into()),
//!             version: "0.1.0".into(),
//!             ..Default::default()
//!         }
//!     }
//! }
//!
//! fn guard_stop(call: HookCall) -> Result<HookResult, PdkError> {
//!     Ok(HookResult::Continue { context: call.context })
//! }
//!
//! plugin! { type: LeaveGuardPlugin }
//! zeitrak_hook! { service: timesheet, command: Stop, phase: Pre, handler: guard_stop }
//! ```

// Hidden re-exports consumed by macro expansions in dependent plugin crates.
// These must be `pub` so `$crate::<path>` resolves correctly when macros expand
// inside those crates.
#[doc(hidden)]
pub use dioxus_extism_pdk::extism_pdk;
#[doc(hidden)]
pub use paste;
#[doc(hidden)]
pub use serde_json;

// Public re-exports: types needed in trait bounds and by plugin authors.
pub use dioxus_extism_pdk::{
    DioxusPlugin,
    EventSubscriber,
    HookCall,
    HookHandler,
    HookRegistration,
    HookResult,
    HostCapability,
    InteractionHandler,
    OnLoad,
    OnUnload,
    PdkError,
    PluginCtx,
    PluginId,
    PluginManifest,
    // View building helpers — re-exported so plugin authors only need zeitrak-plugin-sdk
    PluginView,
    PriorityHint,
    SessionCtx,
    SlotProvider,
    SlotRegistration,
    StateScope,
    TransformProvider,
    ViewBuilder,
    a,
    api_route_fn,
    article,
    aside,
    button,
    div,
    element,
    events_export,
    footer,
    form,
    fragment,
    h1,
    h2,
    h3,
    h4,
    h5,
    h6,
    header,
    hook_export,
    img,
    input,
    interactions_export,
    label,
    li,
    nav,
    ol,
    on_grants_changed_export,
    on_load_export,
    on_unload_export,
    p,
    // re-export the full plugin! macro family so zeitrak plugin authors need only this crate
    plugin,
    section,
    select,
    span,
    table,
    tbody,
    td,
    text,
    textarea,
    th,
    thead,
    tr,
    transform_export,
    ul,
};

use serde::{Deserialize, Serialize};

// ── DomainEventEnvelope ───────────────────────────────────────────────────────

/// Event envelope delivered to a plugin's `on_domain_event` WASM export.
///
/// All string fields use owned `String` values rather than `&'static str` so
/// the type is fully serialisable across the WASM boundary without lifetime
/// constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEventEnvelope {
    /// Aggregate type (e.g. `"timesheet"`, `"activity"`).
    pub aggregate_type: String,
    /// Aggregate identifier (UUID string).
    pub aggregate_id: String,
    /// Event variant name (e.g. `"TimesheetStopped"`).
    pub event_name: String,
    /// Full event payload serialised as JSON.
    pub payload: serde_json::Value,
    /// Caller session at the time the event was raised.
    pub session: SessionCtx,
}

// ── AggregateEventEmit ────────────────────────────────────────────────────────

/// An event emitted by a plugin aggregate's `handle_command` implementation.
///
/// Return one or more of these from [`ZeitrakAggregate::handle_command`] to
/// persist domain events.  The host will call
/// [`ZeitrakAggregate::apply`] once per emitted event after persisting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateEventEmit {
    /// Event type discriminant (e.g. `"LeaveRequestSubmitted"`).
    pub event_type: String,
    /// JSON-encoded event payload.
    pub payload: serde_json::Value,
}

impl AggregateEventEmit {
    /// Construct an event emit with a serialisable payload.
    ///
    /// # Panics
    ///
    /// Panics if `payload` cannot be serialised to JSON.
    #[must_use]
    pub fn new(event_type: impl Into<String>, payload: impl Serialize) -> Self {
        Self {
            event_type: event_type.into(),
            payload: serde_json::to_value(payload)
                .expect("AggregateEventEmit payload must be serialisable"),
        }
    }
}

// ── HandleCommandOutput ───────────────────────────────────────────────────────

/// Return value of [`ZeitrakAggregate::handle_command`].
///
/// The host inspects this value after calling `__handle_command`:
/// - [`HandleCommandOutput::Events`] — persist these events and continue.
/// - [`HandleCommandOutput::Error`] — domain invariant violation; return HTTP 422.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum HandleCommandOutput {
    /// The command was accepted; these events should be persisted and applied.
    Events(Vec<AggregateEventEmit>),
    /// The command was rejected by a domain invariant.
    Error(String),
}

impl HandleCommandOutput {
    /// Convenience constructor: produce a successful result with one event.
    #[must_use]
    pub fn emit(event_type: impl Into<String>, payload: impl Serialize) -> Self {
        Self::Events(vec![AggregateEventEmit::new(event_type, payload)])
    }

    /// Convenience constructor: produce a successful result with multiple events.
    #[must_use]
    pub const fn emit_many(events: Vec<AggregateEventEmit>) -> Self {
        Self::Events(events)
    }

    /// Convenience constructor: reject the command with a message.
    #[must_use]
    pub fn reject(reason: impl Into<String>) -> Self {
        Self::Error(reason.into())
    }
}

// ── ZeitrakAggregate ──────────────────────────────────────────────────────────

/// Implemented by plugin types that own a WASM-backed event-sourced aggregate.
///
/// Implement this trait on a unit struct, then call [`zeitrak_aggregate!`] to
/// generate the three required WASM exports (`__initial_state`, `__apply`,
/// `__handle_command`).
///
/// # Example
///
/// ```ignore
/// use zeitrak_plugin_sdk::*;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// struct LeaveRequestState {
///     status: String,
/// }
///
/// struct LeaveRequest;
///
/// impl ZeitrakAggregate for LeaveRequest {
///     type State = LeaveRequestState;
///
///     fn initial_state() -> Self::State {
///         LeaveRequestState::default()
///     }
///
///     fn apply(mut state: Self::State, event: AggregateEventEmit) -> Result<Self::State, PdkError> {
///         if event.event_type == "LeaveRequestSubmitted" {
///             state.status = "pending".into();
///         }
///         Ok(state)
///     }
///
///     fn handle_command(state: Self::State, cmd: serde_json::Value) -> HandleCommandOutput {
///         HandleCommandOutput::emit("LeaveRequestSubmitted", &cmd)
///     }
/// }
///
/// zeitrak_aggregate! { name: leave_request, handler: LeaveRequest }
/// ```
pub trait ZeitrakAggregate {
    /// The aggregate's mutable state type.
    ///
    /// Must be `Serialize + DeserializeOwned` so it can cross the WASM boundary.
    type State: Serialize + for<'de> Deserialize<'de>;

    /// Return the initial aggregate state (called when the aggregate is first created).
    fn initial_state() -> Self::State;

    /// Fold an event into the current state.
    ///
    /// # Errors
    ///
    /// Return [`PdkError`] to signal an unexpected event; the host will propagate
    /// the error and not persist the event.
    fn apply(state: Self::State, event: AggregateEventEmit) -> Result<Self::State, PdkError>;

    /// Handle a command and produce events or a domain error.
    ///
    /// Return [`HandleCommandOutput::Events`] to emit events, or
    /// [`HandleCommandOutput::Error`] to reject the command.
    fn handle_command(state: Self::State, command: serde_json::Value) -> HandleCommandOutput;
}

// ── PluginProjectionEvent ─────────────────────────────────────────────────────

/// Event envelope forwarded to a plugin's `<projection>__project` WASM export.
///
/// Mirrors the host-side `PluginProjectionEvent` in `zeitrak-plugin-host`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginProjectionEvent {
    /// The stream (aggregate instance) this event belongs to.
    pub stream_id: String,
    /// Event type discriminant.
    pub event_type: String,
    /// JSON-deserialised event payload.
    pub payload: serde_json::Value,
    /// Global position across all streams.
    pub global_position: i64,
    /// Per-stream version number.
    pub version: i64,
}

// ── ZeitrakEventSubscriber ────────────────────────────────────────────────────

/// Implemented by plugins that subscribe to zeitrak domain events.
///
/// List event names in [`PluginManifest::event_subscriptions`] and register
/// the WASM export with [`on_domain_event_export!`].
pub trait ZeitrakEventSubscriber: DioxusPlugin {
    /// Called for every domain event whose name appears in `event_subscriptions`.
    ///
    /// # Errors
    ///
    /// Return [`PdkError`] to signal a processing failure.  The host logs the
    /// error but does not retry or block the event stream.
    fn on_domain_event(envelope: DomainEventEnvelope) -> Result<(), PdkError>;
}

// ── Macros ────────────────────────────────────────────────────────────────────

/// Generate the `on_domain_event` WASM export for a [`ZeitrakEventSubscriber`].
///
/// The host calls this export whenever an event listed in the plugin's
/// `event_subscriptions` fires.
///
/// # Example
///
/// ```ignore
/// on_domain_event_export!(LeaveRequestPlugin);
/// ```
#[macro_export]
macro_rules! on_domain_event_export {
    ($plugin:ty) => {
        #[$crate::extism_pdk::plugin_fn]
        pub fn on_domain_event(
            input: $crate::extism_pdk::Json<$crate::DomainEventEnvelope>,
        ) -> $crate::extism_pdk::FnResult<()> {
            <$plugin as $crate::ZeitrakEventSubscriber>::on_domain_event(input.0)
                .map_err(|e| $crate::extism_pdk::Error::msg(e.to_string()))?;
            Ok(())
        }
    };
}

/// Generate the three WASM exports for a plugin-authored aggregate.
///
/// Requires the `handler` type to implement [`ZeitrakAggregate`].
///
/// Generates:
/// - `<name>__initial_state` — returns the initial state.
/// - `<name>__apply` — folds one event into the current state.
/// - `<name>__handle_command` — produces events or a domain error.
///
/// # Example
///
/// ```ignore
/// struct LeaveRequest;
/// impl ZeitrakAggregate for LeaveRequest { ... }
///
/// zeitrak_aggregate! { name: leave_request, handler: LeaveRequest }
/// ```
#[macro_export]
macro_rules! zeitrak_aggregate {
    (name: $name:ident, handler: $handler:ty $(,)?) => {
        $crate::paste::paste! {
            #[$crate::extism_pdk::plugin_fn]
            pub fn [<$name __initial_state>](
                _input: $crate::extism_pdk::Json<()>,
            ) -> $crate::extism_pdk::FnResult<
                $crate::extism_pdk::Json<<$handler as $crate::ZeitrakAggregate>::State>,
            > {
                Ok($crate::extism_pdk::Json(
                    <$handler as $crate::ZeitrakAggregate>::initial_state(),
                ))
            }

            #[$crate::extism_pdk::plugin_fn]
            pub fn [<$name __apply>](
                input: $crate::extism_pdk::Json<(
                    <$handler as $crate::ZeitrakAggregate>::State,
                    $crate::AggregateEventEmit,
                )>,
            ) -> $crate::extism_pdk::FnResult<
                $crate::extism_pdk::Json<<$handler as $crate::ZeitrakAggregate>::State>,
            > {
                let (state, event) = input.0;
                Ok($crate::extism_pdk::Json(
                    <$handler as $crate::ZeitrakAggregate>::apply(state, event)
                        .map_err(|e: $crate::PdkError| {
                            $crate::extism_pdk::Error::msg(e.to_string())
                        })?,
                ))
            }

            #[$crate::extism_pdk::plugin_fn]
            pub fn [<$name __handle_command>](
                input: $crate::extism_pdk::Json<(
                    <$handler as $crate::ZeitrakAggregate>::State,
                    $crate::serde_json::Value,
                )>,
            ) -> $crate::extism_pdk::FnResult<
                $crate::extism_pdk::Json<$crate::HandleCommandOutput>,
            > {
                let (state, command) = input.0;
                Ok($crate::extism_pdk::Json(
                    <$handler as $crate::ZeitrakAggregate>::handle_command(state, command),
                ))
            }
        }
    };
}

/// Generate the `<name>__project` WASM export for a plugin projection handler.
///
/// The `handler` is any expression of type
/// `fn(PluginProjectionEvent) -> Result<(), PdkError>` (or a closure with the
/// same signature).
///
/// The generated export returns `FnResult<Json<()>>` so the host can
/// distinguish a successful void response (JSON `null`) from an Extism error.
///
/// # Example
///
/// ```ignore
/// fn handle_leave_event(event: PluginProjectionEvent) -> Result<(), PdkError> {
///     // write to plugin storage …
///     Ok(())
/// }
///
/// zeitrak_projection! { name: pending_leaves, handler: handle_leave_event }
/// ```
#[macro_export]
macro_rules! zeitrak_projection {
    (name: $name:ident, handler: $handler:expr $(,)?) => {
        $crate::paste::paste! {
            #[$crate::extism_pdk::plugin_fn]
            pub fn [<$name __project>](
                input: $crate::extism_pdk::Json<$crate::PluginProjectionEvent>,
            ) -> $crate::extism_pdk::FnResult<$crate::extism_pdk::Json<()>> {
                ($handler)(input.0)
                    .map_err(|e: $crate::PdkError| {
                        $crate::extism_pdk::Error::msg(e.to_string())
                    })?;
                Ok($crate::extism_pdk::Json(()))
            }
        }
    };
}

/// Generate a WASM hook export for a zeitrak command.
///
/// Produces the correctly-named WASM export `hook_<service>_<Command>_Pre` or
/// `hook_<service>_<Command>_Post`, matching what the `HookDispatcher` looks
/// for when dispatching pre/post hooks.
///
/// - **Pre** handlers receive [`HookCall`] and return
///   <code>Result<[HookResult], [PdkError]></code>.
///   Return `HookResult::Cancel { reason }` to abort the operation.
/// - **Post** handlers receive [`HookCall`] and return
///   <code>Result<(), [PdkError]></code>.
///   The return value is fire-and-forget; errors are logged by the host.
///
/// The `service` and `command` tokens are identifiers (not string literals).
/// Use lowercase for `service` and title-case for `command` to match the
/// dispatcher's naming convention.
///
/// # Example
///
/// ```ignore
/// fn guard_stop(call: HookCall) -> Result<HookResult, PdkError> {
///     Ok(HookResult::Continue { context: call.context })
/// }
/// zeitrak_hook! { service: timesheet, command: Stop, phase: Pre, handler: guard_stop }
///
/// fn log_stop(call: HookCall) -> Result<(), PdkError> {
///     Ok(())
/// }
/// zeitrak_hook! { service: timesheet, command: Stop, phase: Post, handler: log_stop }
/// ```
#[macro_export]
macro_rules! zeitrak_hook {
    (service: $service:ident, command: $command:ident, phase: Pre, handler: $handler:expr $(,)?) => {
        $crate::paste::paste! {
            #[$crate::extism_pdk::plugin_fn]
            pub fn [<hook_ $service _ $command _Pre>](
                input: $crate::extism_pdk::Json<$crate::HookCall>,
            ) -> $crate::extism_pdk::FnResult<$crate::extism_pdk::Json<$crate::HookResult>> {
                let result: ::core::result::Result<$crate::HookResult, $crate::PdkError> =
                    ($handler)(input.0);
                Ok($crate::extism_pdk::Json(
                    result.map_err(|e| $crate::extism_pdk::Error::msg(e.to_string()))?,
                ))
            }
        }
    };
    (service: $service:ident, command: $command:ident, phase: Post, handler: $handler:expr $(,)?) => {
        $crate::paste::paste! {
            #[$crate::extism_pdk::plugin_fn]
            pub fn [<hook_ $service _ $command _Post>](
                input: $crate::extism_pdk::Json<$crate::HookCall>,
            ) -> $crate::extism_pdk::FnResult<()> {
                ($handler)(input.0)
                    .map_err(|e: $crate::PdkError| $crate::extism_pdk::Error::msg(e.to_string()))?;
                Ok(())
            }
        }
    };
}
