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
pub mod trust;

pub use error::PluginHostError;
pub use host_ctx::{PermissionSet, ZeitrakHostCtx};
pub use trust::{InstallContext, Installer, ZeitrakTrustTier};

use std::sync::Arc;

use dioxus_extism_host::{PluginRuntime, PluginRuntimeError};

use crate::capabilities::build_permission_capability_check;

/// Central facade for the zeitrak plugin system.
///
/// `PluginHost` owns the `dioxus-extism` runtime wired with all zeitrak-specific
/// extension handlers, capability policies, and the domain-event bus. Construct
/// it once at application startup and share it via `Arc`.
pub struct PluginHost {
    runtime: Arc<PluginRuntime<ZeitrakHostCtx>>,
}

impl PluginHost {
    /// Build a new `PluginHost`, registering all zeitrak-specific policies with
    /// the `dioxus-extism` runtime.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying `PluginRuntime` fails to initialise.
    pub async fn new() -> Result<Self, PluginRuntimeError> {
        let runtime = PluginRuntime::<ZeitrakHostCtx>::builder()
            .with_capability_check_ctx(
                "zeitrak.permission",
                build_permission_capability_check(),
            )
            .build()
            .await?;

        Ok(Self { runtime })
    }

    /// Returns a shared reference to the underlying `dioxus-extism` runtime.
    ///
    /// Use this to load, unload, or call plugins.
    #[must_use]
    pub const fn runtime(&self) -> &Arc<PluginRuntime<ZeitrakHostCtx>> {
        &self.runtime
    }
}

impl std::fmt::Debug for PluginHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginHost").finish_non_exhaustive()
    }
}
