//! Typed structs for zeitrak-specific manifest extension values.
//!
//! Each struct corresponds to one `[extensions."zeitrak.*"]` block in a
//! plugin's `plugin.toml` and is deserialized from the opaque `serde_json::Value`
//! that `dioxus-extism` hands to the registered [`ManifestExtensionHandler`]s.

use serde::Deserialize;

// ── zeitrak.app ───────────────────────────────────────────────────────────────

/// `[extensions."zeitrak.app"]`
///
/// ```toml
/// [extensions."zeitrak.app"]
/// min_version = "0.5"
/// ```
#[derive(Debug, Deserialize)]
pub struct ZeitrakAppExtension {
    /// Minimum zeitrak version required by the plugin (semver string).
    pub min_version: String,
}

// ── zeitrak.permissions ───────────────────────────────────────────────────────

/// `[extensions."zeitrak.permissions"]`
///
/// ```toml
/// [extensions."zeitrak.permissions"]
/// contributed = ["leave.submit", "leave.approve"]
/// ```
#[derive(Debug, Deserialize)]
pub struct ZeitrakPermissionsExtension {
    /// Permission names contributed by the plugin.
    ///
    /// These are added to the runtime permission registry and can be granted to
    /// users. Must not conflict with core permission names or use the `admin.`
    /// prefix.
    #[serde(default)]
    pub contributed: Vec<String>,
}
