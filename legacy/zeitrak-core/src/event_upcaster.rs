/// Migrates a persisted event payload from one schema version to the next.
///
/// Each implementation handles exactly one version transition for one event type.
/// Register multiple upcasters in ascending [`source_version`][Self::source_version]
/// order to chain migrations (e.g. V1→V2, V2→V3).
///
/// Implementations must be **pure** (no I/O, no side effects).
pub trait EventUpcaster: Send + Sync {
    /// The event-type discriminator this upcaster applies to
    /// (e.g. `"TimesheetStopped"`, `"plugin.acme.LeaveSubmitted"`).
    fn event_type(&self) -> &str;

    /// The schema version this upcaster reads from.
    fn source_version(&self) -> u32;

    /// The schema version this upcaster produces. Must be strictly greater than
    /// [`source_version()`][Self::source_version].
    fn target_version(&self) -> u32;

    /// Transform the payload from `source_version` to `target_version`.
    ///
    /// # Errors
    ///
    /// Returns [`UpcastError::Migration`] if the payload cannot be transformed.
    fn upcast(&self, payload: serde_json::Value) -> Result<serde_json::Value, UpcastError>;
}

/// Error returned by [`EventUpcaster::upcast`].
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UpcastError {
    /// The payload could not be migrated to the next schema version.
    #[error("payload migration failed: {0}")]
    Migration(String),
    /// A JSON serialisation error occurred during migration.
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
}
