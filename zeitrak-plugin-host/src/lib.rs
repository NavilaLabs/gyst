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

pub mod error;

pub use error::PluginHostError;

/// Central facade for the zeitrak plugin system.
///
/// `PluginHost` owns the `dioxus-extism` runtime wired with all zeitrak-specific
/// extension handlers, capability policies, and the domain-event bus. Construct
/// it once at application startup and share it via `Arc`.
///
/// Fields are added incrementally as sub-systems are introduced in Phase B–H.
#[derive(Debug)]
pub struct PluginHost {
    _priv: (),
}

impl PluginHost {
    /// Creates a new, unconfigured `PluginHost`.
    ///
    /// This constructor will grow as sub-systems are added in subsequent phases.
    /// For now it returns a shell that can be held in `Arc<PluginHost>`.
    #[must_use]
    pub const fn new() -> Self {
        Self { _priv: () }
    }
}

impl Default for PluginHost {
    fn default() -> Self {
        Self::new()
    }
}
