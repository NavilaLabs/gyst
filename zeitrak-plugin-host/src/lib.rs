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
pub mod host_ctx;
pub mod manifest;
pub mod manifest_handlers;
pub mod trust;

pub use error::PluginHostError;
pub use host_ctx::{PermissionSet, ZeitrakHostCtx};
pub use trust::{InstallContext, Installer, ZeitrakTrustTier};

use std::collections::HashSet;
use std::sync::{Arc, RwLock};

use dioxus_extism_host::{PluginRuntime, PluginRuntimeError};

use crate::capabilities::build_permission_capability_check;
use crate::manifest_handlers::{ZeitrakAppHandler, ZeitrakPermissionsHandler};

/// Central facade for the zeitrak plugin system.
///
/// `PluginHost` owns the `dioxus-extism` runtime wired with all zeitrak-specific
/// extension handlers, capability policies, and the domain-event bus. Construct
/// it once at application startup and share it via `Arc`.
pub struct PluginHost {
    runtime: Arc<PluginRuntime<ZeitrakHostCtx>>,
    /// Permissions contributed by loaded plugins, registered during `on_load`.
    contributed_permissions: Arc<RwLock<HashSet<String>>>,
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

        let runtime = PluginRuntime::<ZeitrakHostCtx>::builder()
            .with_manifest_extension(
                "zeitrak.app",
                Arc::new(ZeitrakAppHandler),
            )
            .with_manifest_extension(
                "zeitrak.permissions",
                Arc::new(ZeitrakPermissionsHandler::new(Arc::clone(
                    &contributed_permissions,
                ))),
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
}

impl std::fmt::Debug for PluginHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginHost").finish_non_exhaustive()
    }
}
