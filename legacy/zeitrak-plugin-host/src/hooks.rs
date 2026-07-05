//! Hook registry — stores command hooks contributed by loaded plugins.
//!
//! Hooks are registered during `on_load` of the `zeitrak.hooks` manifest
//! extension and deregistered on `on_unload`. Phase D (§8.4) wires the
//! actual dispatch path; this module only manages the registry state.

use crate::manifest::HookPhase;

/// A registered command hook entry, produced when a plugin is loaded with a
/// `zeitrak.hooks` manifest extension.
#[derive(Debug, Clone)]
pub struct RegisteredHook {
    /// Plugin that owns this hook.
    pub plugin_id: String,
    /// Service targeted by the hook (e.g. `"timesheet"`).
    pub service: String,
    /// Command targeted by the hook (e.g. `"Stop"`).
    pub command: String,
    /// Whether this hook runs before or after the command.
    pub phase: HookPhase,
    /// Dispatch priority. Lower values run first; ties are broken by
    /// `plugin_id` lexicographic order.
    pub priority: i32,
}

/// Shared registry of all plugin-registered command hooks.
///
/// Ordering within a `(service, command, phase)` group is by `priority`
/// ascending, ties broken by `plugin_id` lexicographic order — matching the
/// ordering rule from §8.4.
///
/// Phase D wires the dispatch path that reads from this registry; Phase C
/// only populates it.
#[derive(Debug, Default)]
pub struct HookRegistry {
    hooks: Vec<RegisteredHook>,
}

impl HookRegistry {
    /// Return a new, empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register all hooks contributed by a plugin.
    pub fn register(&mut self, hooks: impl IntoIterator<Item = RegisteredHook>) {
        self.hooks.extend(hooks);
    }

    /// Remove all hooks registered by `plugin_id`.
    pub fn unregister(&mut self, plugin_id: &str) {
        self.hooks.retain(|h| h.plugin_id != plugin_id);
    }

    /// Returns all hooks matching `(service, command, phase)`, sorted by
    /// priority ascending then `plugin_id` lexicographic.
    #[must_use]
    pub fn lookup(&self, service: &str, command: &str, phase: &HookPhase) -> Vec<&RegisteredHook> {
        let mut matching: Vec<&RegisteredHook> = self
            .hooks
            .iter()
            .filter(|h| h.service == service && h.command == command && h.phase == *phase)
            .collect();
        matching.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| a.plugin_id.cmp(&b.plugin_id))
        });
        matching
    }

    /// Returns the total number of registered hooks.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.hooks.len()
    }

    /// Returns `true` if no hooks are registered.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_hook(
        plugin_id: &str,
        service: &str,
        command: &str,
        phase: HookPhase,
        priority: i32,
    ) -> RegisteredHook {
        RegisteredHook {
            plugin_id: plugin_id.to_string(),
            service: service.to_string(),
            command: command.to_string(),
            phase,
            priority,
        }
    }

    #[test]
    fn lookup_returns_hooks_sorted_by_priority_then_plugin_id() {
        let mut reg = HookRegistry::new();
        reg.register([
            make_hook("plugin-b", "timesheet", "Stop", HookPhase::Pre, 100),
            make_hook("plugin-a", "timesheet", "Stop", HookPhase::Pre, 100),
            make_hook("plugin-c", "timesheet", "Stop", HookPhase::Pre, 50),
        ]);
        let hooks = reg.lookup("timesheet", "Stop", &HookPhase::Pre);
        assert_eq!(hooks.len(), 3);
        assert_eq!(hooks[0].plugin_id, "plugin-c");
        assert_eq!(hooks[1].plugin_id, "plugin-a");
        assert_eq!(hooks[2].plugin_id, "plugin-b");
    }

    #[test]
    fn lookup_filters_by_phase() {
        let mut reg = HookRegistry::new();
        reg.register([
            make_hook("plugin-a", "timesheet", "Stop", HookPhase::Pre, 100),
            make_hook("plugin-b", "timesheet", "Stop", HookPhase::Post, 100),
        ]);
        let pre = reg.lookup("timesheet", "Stop", &HookPhase::Pre);
        let post = reg.lookup("timesheet", "Stop", &HookPhase::Post);
        assert_eq!(pre.len(), 1);
        assert_eq!(post.len(), 1);
        assert_eq!(pre[0].plugin_id, "plugin-a");
        assert_eq!(post[0].plugin_id, "plugin-b");
    }

    #[test]
    fn unregister_removes_all_hooks_for_plugin() {
        let mut reg = HookRegistry::new();
        reg.register([
            make_hook("plugin-a", "timesheet", "Stop", HookPhase::Pre, 100),
            make_hook("plugin-a", "activity", "Create", HookPhase::Post, 50),
            make_hook("plugin-b", "timesheet", "Stop", HookPhase::Pre, 200),
        ]);
        reg.unregister("plugin-a");
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.hooks[0].plugin_id, "plugin-b");
    }
}
