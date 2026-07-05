//! Per-trust-tier quota defaults (Phase H — §14, step 34).
//!
//! Each [`ZeitrakTrustTier`] implies a different risk profile, so plugins in
//! lower tiers receive tighter resource limits.  Hosts pass the returned
//! [`PluginInstallConfig`] to `PluginRuntime::install_plugin` (or
//! `install_bundle`) when no caller-supplied overrides are present.
//!
//! | Tier            | Pool slots | Call timeout | Max fuel (Wasm ops) |
//! |-----------------|-----------|--------------|---------------------|
//! | `Tenant`        | 1         | 5 s          | 10 M                |
//! | `Instance`      | 2         | 10 s         | 100 M               |
//! | `SignedInstance` | 4        | 30 s         | unlimited           |

use std::time::Duration;

use dioxus_extism_host::PluginInstallConfig;

use crate::trust::ZeitrakTrustTier;

impl ZeitrakTrustTier {
    /// Build a [`PluginInstallConfig`] with the default quotas for this tier.
    ///
    /// Callers may override individual fields after calling this method.
    #[must_use]
    pub fn default_install_config(&self) -> PluginInstallConfig {
        match self {
            Self::Tenant => PluginInstallConfig {
                pool_size: Some(1),
                max_call_duration: Some(Duration::from_secs(5)),
                max_fuel: Some(10_000_000),
                ..PluginInstallConfig::default()
            },
            Self::Instance => PluginInstallConfig {
                pool_size: Some(2),
                max_call_duration: Some(Duration::from_secs(10)),
                max_fuel: Some(100_000_000),
                ..PluginInstallConfig::default()
            },
            Self::SignedInstance => PluginInstallConfig {
                pool_size: Some(4),
                max_call_duration: Some(Duration::from_secs(30)),
                max_fuel: None,
                ..PluginInstallConfig::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenant_is_most_restrictive() {
        let tenant = ZeitrakTrustTier::Tenant.default_install_config();
        let instance = ZeitrakTrustTier::Instance.default_install_config();
        let signed = ZeitrakTrustTier::SignedInstance.default_install_config();

        assert!(tenant.pool_size.unwrap() <= instance.pool_size.unwrap());
        assert!(instance.pool_size.unwrap() <= signed.pool_size.unwrap());
        assert!(tenant.max_call_duration.unwrap() <= instance.max_call_duration.unwrap());
        assert!(instance.max_call_duration.unwrap() <= signed.max_call_duration.unwrap());
        assert!(tenant.max_fuel.is_some());
        assert!(instance.max_fuel.is_some());
        assert!(signed.max_fuel.is_none(), "signed tier has no fuel cap");
    }
}
