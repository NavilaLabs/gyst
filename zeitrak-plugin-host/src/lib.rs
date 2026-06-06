//! `zeitrak-plugin-host` — zeitrak-specific plugin runtime.
//!
//! This crate bridges [dioxus-extism](dioxus_extism_host)'s host-agnostic plugin
//! primitives with zeitrak's domain model: event-sourced aggregates, domain-event
//! bus, application-service hooks, trust tiers, capability gating, and plugin
//! storage. `dioxus-extism` itself gains no zeitrak vocabulary — all zeitrak-specific
//! semantics are confined here.
//!
//! # Crate position in the onion
//!
//! ```text
//! zeitrak-core
//!     ↑
//! zeitrak-infrastructure (port traits)
//!     ↑
//! zeitrak-infrastructure-impl (adapters)
//!     ↑
//! zeitrak-plugin-host          ← this crate
//!     ↑
//! zeitrak (facade)
//! ```

pub mod capabilities;
pub mod error;
pub mod event_bus;
pub mod hooks;
pub mod host_ctx;
pub mod manifest;
pub mod manifest_handlers;
pub mod registries;
pub mod trust;

pub use error::PluginHostError;
pub use host_ctx::{PermissionSet, ZeitrakHostCtx};
pub use trust::{InstallContext, Installer, ZeitrakTrustTier};

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use dioxus_extism_host::{PluginRuntime, PluginRuntimeError};

use crate::capabilities::build_permission_capability_check;
use crate::hooks::HookRegistry;
use crate::manifest_handlers::{
    ZeitrakAggregatesHandler, ZeitrakAppHandler, ZeitrakEventsHandler, ZeitrakHooksHandler,
    ZeitrakPermissionsHandler, ZeitrakProjectionsHandler, CORE_DOMAIN_EVENTS,
};
use crate::registries::{AggregateRegistry, ProjectionRegistry};

/// Central facade for the zeitrak plugin system.
///
/// `PluginHost` owns the `dioxus-extism` runtime wired with all zeitrak-specific
/// extension handlers, capability policies, and the domain-event bus. Construct
/// it once at application startup and share it via `Arc`.
pub struct PluginHost {
    runtime: Arc<PluginRuntime<ZeitrakHostCtx>>,
    /// Permissions contributed by loaded plugins via `zeitrak.permissions`.
    contributed_permissions: Arc<RwLock<HashSet<String>>>,
    /// Known domain event names: core events + plugin-contributed (step 13).
    known_events: Arc<RwLock<HashSet<String>>>,
    /// Plugin → subscribed event names, populated via `zeitrak.events`.
    event_subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Command hooks registered by plugins via `zeitrak.hooks`.
    hook_registry: Arc<RwLock<HookRegistry>>,
    /// Plugin-contributed aggregate types, registered via `zeitrak.aggregates`.
    aggregate_registry: Arc<RwLock<AggregateRegistry>>,
    /// Plugin-contributed projections, registered via `zeitrak.projections`.
    projection_registry: Arc<RwLock<ProjectionRegistry>>,
}

impl PluginHost {
    /// Build a new `PluginHost`, registering all zeitrak-specific policies with
    /// the `dioxus-extism` runtime.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying `PluginRuntime` fails to initialise.
    pub async fn new() -> Result<Self, PluginRuntimeError> {
        let contributed_permissions: Arc<RwLock<HashSet<String>>> =
            Arc::new(RwLock::new(HashSet::new()));

        let known_events: Arc<RwLock<HashSet<String>>> = Arc::new(RwLock::new(
            CORE_DOMAIN_EVENTS.iter().map(|s| (*s).to_string()).collect(),
        ));

        let event_subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>> =
            Arc::new(RwLock::new(HashMap::new()));

        let hook_registry: Arc<RwLock<HookRegistry>> =
            Arc::new(RwLock::new(HookRegistry::new()));

        let aggregate_registry: Arc<RwLock<AggregateRegistry>> =
            Arc::new(RwLock::new(AggregateRegistry::new()));

        let projection_registry: Arc<RwLock<ProjectionRegistry>> =
            Arc::new(RwLock::new(ProjectionRegistry::new()));

        let runtime = PluginRuntime::<ZeitrakHostCtx>::builder()
            .with_manifest_extension("zeitrak.app", Arc::new(ZeitrakAppHandler))
            .with_manifest_extension(
                "zeitrak.permissions",
                Arc::new(ZeitrakPermissionsHandler::new(Arc::clone(
                    &contributed_permissions,
                ))),
            )
            .with_manifest_extension(
                "zeitrak.events",
                Arc::new(ZeitrakEventsHandler::new(
                    Arc::clone(&known_events),
                    Arc::clone(&event_subscriptions),
                )),
            )
            .with_manifest_extension(
                "zeitrak.hooks",
                Arc::new(ZeitrakHooksHandler::new(Arc::clone(&hook_registry))),
            )
            .with_manifest_extension(
                "zeitrak.aggregates",
                Arc::new(ZeitrakAggregatesHandler::new(Arc::clone(&aggregate_registry))),
            )
            .with_manifest_extension(
                "zeitrak.projections",
                Arc::new(ZeitrakProjectionsHandler::new(Arc::clone(&projection_registry))),
            )
            .with_capability_check_ctx(
                "zeitrak.permission",
                build_permission_capability_check(),
            )
            .build()
            .await?;

        Ok(Self {
            runtime,
            contributed_permissions,
            known_events,
            event_subscriptions,
            hook_registry,
            aggregate_registry,
            projection_registry,
        })
    }

    /// Returns a shared reference to the underlying `dioxus-extism` runtime.
    ///
    /// Use this to load, unload, or call plugins.
    #[must_use]
    pub const fn runtime(&self) -> &Arc<PluginRuntime<ZeitrakHostCtx>> {
        &self.runtime
    }

    /// Returns the shared set of permission names contributed by loaded plugins.
    ///
    /// The set grows as plugins are loaded via `zeitrak.permissions` extensions.
    #[must_use]
    pub fn contributed_permissions(&self) -> Arc<RwLock<HashSet<String>>> {
        Arc::clone(&self.contributed_permissions)
    }

    /// Returns the shared set of known domain event names.
    ///
    /// Pre-populated with core event names; extended by `ZeitrakAggregatesHandler`
    /// (step 13) as plugin aggregates are registered.
    #[must_use]
    pub fn known_events(&self) -> Arc<RwLock<HashSet<String>>> {
        Arc::clone(&self.known_events)
    }

    /// Returns the shared plugin → subscribed-event-names map.
    ///
    /// Phase D (§8.3) reads this map to route broadcast events to plugins.
    #[must_use]
    pub fn event_subscriptions(&self) -> Arc<RwLock<HashMap<String, Vec<String>>>> {
        Arc::clone(&self.event_subscriptions)
    }

    /// Returns the shared command hook registry.
    ///
    /// Phase D (§8.4) reads this registry when dispatching pre/post hooks.
    #[must_use]
    pub fn hook_registry(&self) -> Arc<RwLock<HookRegistry>> {
        Arc::clone(&self.hook_registry)
    }

    /// Returns the shared aggregate registry.
    ///
    /// Phase E (§9.2) builds WASM-backed runtime wrappers from these entries.
    #[must_use]
    pub fn aggregate_registry(&self) -> Arc<RwLock<AggregateRegistry>> {
        Arc::clone(&self.aggregate_registry)
    }

    /// Returns the shared projection registry.
    ///
    /// Phase E (§9.5) wires each projection into the `eventually-projection`
    /// runner; Phase F (§10) creates the backing SQL tables.
    #[must_use]
    pub fn projection_registry(&self) -> Arc<RwLock<ProjectionRegistry>> {
        Arc::clone(&self.projection_registry)
    }
}

impl std::fmt::Debug for PluginHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginHost").finish_non_exhaustive()
    }
}
