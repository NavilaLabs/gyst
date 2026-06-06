/// Top-level error type for `zeitrak-plugin-host`.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PluginHostError {
    /// A plugin manifest extension failed validation or load.
    #[error("manifest extension error: {0}")]
    ManifestExtension(String),

    /// A plugin's declared capability was denied.
    #[error("capability denied for plugin `{plugin_id}`: {reason}")]
    CapabilityDenied { plugin_id: String, reason: String },

    /// Table access was denied — the plugin attempted to reference a table
    /// outside its `plugin_<id>__` namespace.
    #[error("table access denied for plugin `{plugin_id}`: {table}")]
    TableAccessDenied { plugin_id: String, table: String },

    /// The requested plugin is not currently loaded.
    #[error("plugin `{0}` is not loaded")]
    PluginNotLoaded(String),

    /// A plugin WASM invocation returned an error payload.
    #[error("plugin `{plugin_id}` invocation `{function}` failed: {message}")]
    Invocation {
        plugin_id: String,
        function: String,
        message: String,
    },

    /// An error from the underlying `dioxus-extism-host` runtime.
    #[error("runtime error: {0}")]
    Runtime(#[from] dioxus_extism_host::PluginRuntimeError),

    /// A database or event-store error.
    #[error("storage error: {0}")]
    Storage(#[from] sqlx::Error),

    /// A serialisation error.
    #[error("serialisation error: {0}")]
    Serde(#[from] serde_json::Error),
}
