//! Aggregate and projection registries for plugin-contributed domain types.
//!
//! These registries are populated in Phase C (manifest-extension handlers)
//! and consumed in Phase E (runtime delegation to WASM exports).  Tables are
//! never dropped on plugin unload — only on explicit uninstall with
//! `drop_tables: true`.

use crate::manifest::{AggregateDecl, ProjectionDecl};

// ── Aggregate registry ────────────────────────────────────────────────────────

/// A plugin-contributed aggregate, as recorded at load time.
#[derive(Debug, Clone)]
pub struct RegisteredAggregate {
    /// Plugin that declared this aggregate.
    pub plugin_id: String,
    /// The manifest declaration.
    pub decl: AggregateDecl,
}

/// Registry of all plugin-contributed aggregate types.
///
/// Aggregate names are globally unique across all plugins — duplicate names
/// are rejected at validation time.  Phase E (§9.2) builds WASM-backed
/// `Aggregate` wrappers from these entries.
#[derive(Debug, Default)]
pub struct AggregateRegistry {
    entries: Vec<RegisteredAggregate>,
}

impl AggregateRegistry {
    /// Return a new, empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if `name` is already registered by any plugin.
    #[must_use]
    pub fn contains_name(&self, name: &str) -> bool {
        self.entries.iter().any(|a| a.decl.name == name)
    }

    /// Register a new aggregate.
    pub fn register(&mut self, aggregate: RegisteredAggregate) {
        self.entries.push(aggregate);
    }

    /// Remove all aggregates registered by `plugin_id`.
    pub fn unregister(&mut self, plugin_id: &str) {
        self.entries.retain(|a| a.plugin_id != plugin_id);
    }

    /// Iterate over all registered aggregates.
    pub fn iter(&self) -> impl Iterator<Item = &RegisteredAggregate> {
        self.entries.iter()
    }

    /// Returns the total number of registered aggregates.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if no aggregates are registered.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ── Projection registry ───────────────────────────────────────────────────────

/// A plugin-contributed projection, as recorded at load time.
#[derive(Debug, Clone)]
pub struct RegisteredProjection {
    /// Plugin that declared this projection.
    pub plugin_id: String,
    /// The manifest declaration.
    pub decl: ProjectionDecl,
}

/// Registry of all plugin-contributed projections.
///
/// Projection names and table names are globally unique — duplicates are
/// rejected at validation time.  Phase F (§10) creates the backing SQL tables;
/// Phase E (§9.5) wires the projector into `eventually-projection`'s runner.
#[derive(Debug, Default)]
pub struct ProjectionRegistry {
    entries: Vec<RegisteredProjection>,
}

impl ProjectionRegistry {
    /// Return a new, empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if `name` is already registered by any plugin.
    #[must_use]
    pub fn contains_name(&self, name: &str) -> bool {
        self.entries.iter().any(|p| p.decl.name == name)
    }

    /// Returns `true` if `table` is already in use by any plugin.
    #[must_use]
    pub fn contains_table(&self, table: &str) -> bool {
        self.entries.iter().any(|p| p.decl.table == table)
    }

    /// Register a new projection.
    pub fn register(&mut self, projection: RegisteredProjection) {
        self.entries.push(projection);
    }

    /// Remove all projections registered by `plugin_id`.
    pub fn unregister(&mut self, plugin_id: &str) {
        self.entries.retain(|p| p.plugin_id != plugin_id);
    }

    /// Iterate over all registered projections.
    pub fn iter(&self) -> impl Iterator<Item = &RegisteredProjection> {
        self.entries.iter()
    }

    /// Returns the total number of registered projections.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if no projections are registered.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::CommandDecl;

    fn make_aggregate_decl(name: &str) -> AggregateDecl {
        AggregateDecl {
            name: name.to_string(),
            events: vec!["Created".to_string()],
            snapshot_every: None,
            commands: vec![CommandDecl {
                name: "Create".to_string(),
                permission: "leave.create".to_string(),
            }],
        }
    }

    fn make_projection_decl(name: &str, table: &str) -> ProjectionDecl {
        ProjectionDecl {
            name: name.to_string(),
            table: table.to_string(),
            events: vec!["Created".to_string()],
        }
    }

    #[test]
    fn aggregate_registry_detects_name_collision() {
        let mut reg = AggregateRegistry::new();
        reg.register(RegisteredAggregate {
            plugin_id: "plugin-a".to_string(),
            decl: make_aggregate_decl("leave_request"),
        });
        assert!(reg.contains_name("leave_request"));
        assert!(!reg.contains_name("approval"));
    }

    #[test]
    fn aggregate_registry_unregister_removes_entries() {
        let mut reg = AggregateRegistry::new();
        reg.register(RegisteredAggregate {
            plugin_id: "plugin-a".to_string(),
            decl: make_aggregate_decl("leave_request"),
        });
        reg.register(RegisteredAggregate {
            plugin_id: "plugin-b".to_string(),
            decl: make_aggregate_decl("expense_report"),
        });
        reg.unregister("plugin-a");
        assert_eq!(reg.len(), 1);
        assert!(!reg.contains_name("leave_request"));
        assert!(reg.contains_name("expense_report"));
    }

    #[test]
    fn projection_registry_detects_name_and_table_collision() {
        let mut reg = ProjectionRegistry::new();
        reg.register(RegisteredProjection {
            plugin_id: "plugin-a".to_string(),
            decl: make_projection_decl("pending_leaves", "pending"),
        });
        assert!(reg.contains_name("pending_leaves"));
        assert!(reg.contains_table("pending"));
        assert!(!reg.contains_name("other"));
        assert!(!reg.contains_table("other_table"));
    }

    #[test]
    fn projection_registry_unregister_removes_entries() {
        let mut reg = ProjectionRegistry::new();
        reg.register(RegisteredProjection {
            plugin_id: "plugin-a".to_string(),
            decl: make_projection_decl("pending_leaves", "pending"),
        });
        reg.unregister("plugin-a");
        assert!(reg.is_empty());
    }
}
