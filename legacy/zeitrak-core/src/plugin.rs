/// The single contract every Zeitrak plugin must implement.
///
/// Plugins register permissions and provide metadata.  Infrastructure-level
/// capabilities (projectors, event handlers) are wired up separately in
/// `zeitrak-infrastructure-impl` so that `zeitrak-core` stays free of I/O
/// dependencies.
pub trait ZeitrakPlugin: Send + Sync {
    /// Stable reverse-DNS identifier, e.g. `"com.example.my-plugin"`.
    fn id(&self) -> &'static str;

    /// Semver-compatible version string, e.g. `"1.0.0"`.
    fn version(&self) -> &'static str;

    /// Additional permissions this plugin defines.
    ///
    /// Returned strings are merged into the global [`PermissionsRegistry`] and
    /// seeded into the database on first run.
    ///
    /// [`PermissionsRegistry`]: crate::permissions::PermissionsRegistry
    fn permissions(&self) -> &[&'static str] {
        &[]
    }
}

/// Manages all registered plugins and exposes their combined capabilities.
#[derive(Default)]
pub struct PluginRegistry {
    plugins: Vec<Box<dyn ZeitrakPlugin>>,
}

impl PluginRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, plugin: impl ZeitrakPlugin + 'static) {
        self.plugins.push(Box::new(plugin));
    }

    /// Returns an iterator over every permission contributed by all plugins.
    pub fn all_permissions(&self) -> impl Iterator<Item = &str> {
        self.plugins
            .iter()
            .flat_map(|p| p.permissions().iter().copied())
    }
}
