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

// Public re-exports: types needed in trait bounds and by plugin authors.
pub use dioxus_extism_pdk::{
    DioxusPlugin, HookCall, HookResult, PdkError, PluginCtx, PluginManifest, SessionCtx,
    // re-export the full plugin! macro family so zeitrak plugin authors need only this crate
    plugin, on_load_export, on_unload_export, on_grants_changed_export,
    hook_export, transform_export, events_export, interactions_export, api_route_fn,
    PluginId, SlotRegistration, HookRegistration, HostCapability, StateScope,
    SlotProvider, HookHandler, EventSubscriber, InteractionHandler, OnLoad, OnUnload,
    TransformProvider,
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
                .map_err(|e| $crate::extism_pdk::Error::msg(e.to_string()))
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
                    .map_err(|e: $crate::PdkError| $crate::extism_pdk::Error::msg(e.to_string()))
            }
        }
    };
}
