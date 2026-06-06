//! Plugin-facing domain event bus.
//!
//! [`DomainEvent`] is a `serde`-serialisable enum that exposes all core
//! zeitrak aggregate events plus a catch-all [`DomainEvent::Plugin`] variant
//! for events produced by plugin-authored aggregates.
//!
//! [`PluginEventBus`] is backed by a `tokio::sync::broadcast::Sender` so that
//! slow plugin receivers do not block domain operations.  Events are published
//! after a successful aggregate save (Phase D step 15); plugins subscribe via
//! the `zeitrak.events` manifest extension (Phase C step 12).
//!
//! # Crate position
//!
//! `PluginEventBus` lives in `zeitrak-plugin-host` and implements
//! `zeitrak_core::shared::event_bus::DomainEventHandler` so it can be
//! registered with the core `EventBus` that the save-path wrapper calls.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use eventually::message::Message as _;
use serde::Serialize;
use zeitrak_core::admin::invitation::InvitationEvent;
use zeitrak_core::admin::permission::PermissionEvent;
use zeitrak_core::admin::user::UserEvent;
use zeitrak_core::admin::workspace::WorkspaceEvent;
use zeitrak_core::admin::workspace_role::WorkspaceRoleEvent;
use zeitrak_core::shared::event_bus::{DomainEventEnvelope, DomainEventHandler};
use zeitrak_core::tenant::activity::ActivityEvent;
use zeitrak_core::tenant::timesheet::TimesheetEvent;
use zeitrak_core::tenant::timesheet_tag::TimesheetTagEvent;

// ── DomainEvent ───────────────────────────────────────────────────────────────

/// All zeitrak domain events, usable by plugins.
///
/// Variants correspond 1-to-1 with core aggregate event enums.  The
/// `#[non_exhaustive]` attribute ensures future core events do not
/// silently break plugin code.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub enum DomainEvent {
    /// An event from the `activity` aggregate.
    Activity(ActivityEvent),
    /// An event from the `timesheet` aggregate.
    Timesheet(TimesheetEvent),
    /// An event from the `timesheet_tag` aggregate.
    TimesheetTag(TimesheetTagEvent),
    /// An event from the `user` aggregate.
    User(UserEvent),
    /// An event from the `workspace` aggregate.
    Workspace(WorkspaceEvent),
    /// An event from the `workspace_role` aggregate.
    WorkspaceRole(WorkspaceRoleEvent),
    /// An event from the `invitation` aggregate.
    Invitation(InvitationEvent),
    /// An event from the `permission` aggregate.
    Permission(PermissionEvent),
    /// An event produced by a plugin-authored aggregate.
    Plugin {
        /// The plugin that owns the aggregate.
        plugin_id: String,
        /// The aggregate type name (matches `AggregateDecl::name`).
        aggregate: String,
        /// The event variant name.
        event_type: String,
        /// The event payload, serialised by the plugin.
        payload: serde_json::Value,
    },
}

impl DomainEvent {
    /// Returns the event name string.
    ///
    /// For core events, this matches the string returned by `Message::name()`
    /// on the inner event type (e.g. `"ActivityCreated"`).
    #[must_use]
    pub fn event_name(&self) -> &str {
        match self {
            Self::Activity(e) => e.name(),
            Self::Timesheet(e) => e.name(),
            Self::TimesheetTag(e) => e.name(),
            Self::User(e) => e.name(),
            Self::Workspace(e) => e.name(),
            Self::WorkspaceRole(e) => e.name(),
            Self::Invitation(e) => e.name(),
            Self::Permission(e) => e.name(),
            Self::Plugin { event_type, .. } => event_type.as_str(),
        }
    }
}

// ── PluginEventBus ────────────────────────────────────────────────────────────

/// Converts a [`DomainEventEnvelope`] (aggregate-type + JSON payload) into the
/// strongly-typed [`DomainEvent`].
///
/// Returns `None` for aggregate types that are not yet mapped (e.g. unknown
/// plugin aggregates that arrive before their manifest is processed — very
/// rare; they are published as `DomainEvent::Plugin` with an empty
/// `plugin_id`).
fn envelope_to_domain_event(env: &DomainEventEnvelope) -> Option<DomainEvent> {
    match env.aggregate_type {
        "activity" => {
            serde_json::from_value(env.payload.clone())
                .ok()
                .map(DomainEvent::Activity)
        }
        "timesheet" => {
            serde_json::from_value(env.payload.clone())
                .ok()
                .map(DomainEvent::Timesheet)
        }
        "timesheet_tag" => {
            serde_json::from_value(env.payload.clone())
                .ok()
                .map(DomainEvent::TimesheetTag)
        }
        "user" => {
            serde_json::from_value(env.payload.clone())
                .ok()
                .map(DomainEvent::User)
        }
        "workspace" => {
            serde_json::from_value(env.payload.clone())
                .ok()
                .map(DomainEvent::Workspace)
        }
        "workspace_role" => {
            serde_json::from_value(env.payload.clone())
                .ok()
                .map(DomainEvent::WorkspaceRole)
        }
        "invitation" => {
            serde_json::from_value(env.payload.clone())
                .ok()
                .map(DomainEvent::Invitation)
        }
        "permission" => {
            serde_json::from_value(env.payload.clone())
                .ok()
                .map(DomainEvent::Permission)
        }
        _ => Some(DomainEvent::Plugin {
            plugin_id: String::new(),
            aggregate: env.aggregate_type.to_string(),
            event_type: env.event_name.to_string(),
            payload: env.payload.clone(),
        }),
    }
}

/// The default broadcast channel capacity.
///
/// Lagging subscribers drop messages rather than blocking domain operations.
pub const DEFAULT_BUS_CAPACITY: usize = 1024;

/// A broadcast-based event bus for plugin event subscriptions.
///
/// `PluginEventBus` implements [`DomainEventHandler`] so it can be registered
/// with the core `EventBus` in the save-path wrapper (step 15).  When an event
/// arrives it is converted to [`DomainEvent`] and sent to all active
/// subscribers.  Slow receivers drop messages on lag — domain operations are
/// never blocked.
#[derive(Clone)]
pub struct PluginEventBus {
    sender: tokio::sync::broadcast::Sender<DomainEvent>,
    /// Plugin → subscribed event names (maintained by `ZeitrakEventsHandler`).
    subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl PluginEventBus {
    /// Create a new bus with the given broadcast channel capacity.
    #[must_use]
    pub fn new(
        capacity: usize,
        subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>,
    ) -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(capacity);
        Self {
            sender,
            subscriptions,
        }
    }

    /// Subscribe to the broadcast stream.
    ///
    /// The returned receiver will miss events if it falls behind by more than
    /// `capacity` (the channel capacity passed to [`PluginEventBus::new`]).
    #[must_use]
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<DomainEvent> {
        self.sender.subscribe()
    }

    /// Publish `event` to all active receivers.
    ///
    /// Returns `true` if at least one receiver was active; returns `false`
    /// when no receivers exist (event is discarded).
    #[must_use]
    pub fn publish(&self, event: DomainEvent) -> bool {
        self.sender.send(event).is_ok()
    }

    /// Returns all plugin IDs that have subscribed to `event_name`.
    #[must_use]
    pub fn subscribers_for(&self, event_name: &str) -> Vec<String> {
        let Ok(subs) = self.subscriptions.read() else {
            return vec![];
        };
        subs.iter()
            .filter(|(_, events)| events.iter().any(|e| e == event_name))
            .map(|(id, _)| id.clone())
            .collect()
    }
}

impl std::fmt::Debug for PluginEventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginEventBus")
            .field("receiver_count", &self.sender.receiver_count())
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl DomainEventHandler for PluginEventBus {
    async fn on_event(
        &self,
        envelope: &DomainEventEnvelope,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let Some(event) = envelope_to_domain_event(envelope) else {
            tracing::debug!(
                aggregate_type = envelope.aggregate_type,
                event_name = envelope.event_name,
                "PluginEventBus: unmapped event type, skipping"
            );
            return Ok(());
        };
        let _active = self.publish(event);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_subs() -> Arc<RwLock<HashMap<String, Vec<String>>>> {
        Arc::new(RwLock::new(HashMap::new()))
    }

    #[test]
    fn event_name_returns_correct_string_for_plugin_variant() {
        let ev = DomainEvent::Plugin {
            plugin_id: "test".to_string(),
            aggregate: "leave_request".to_string(),
            event_type: "Submitted".to_string(),
            payload: serde_json::Value::Null,
        };
        assert_eq!(ev.event_name(), "Submitted");
    }

    #[test]
    fn subscribers_for_returns_matching_plugins() {
        let subs = make_subs();
        subs.write().unwrap().insert(
            "plugin-a".to_string(),
            vec!["TimesheetStopped".to_string(), "ActivityCreated".to_string()],
        );
        subs.write().unwrap().insert(
            "plugin-b".to_string(),
            vec!["ActivityCreated".to_string()],
        );

        let bus = PluginEventBus::new(DEFAULT_BUS_CAPACITY, Arc::clone(&subs));
        let mut result = bus.subscribers_for("ActivityCreated");
        result.sort();
        assert_eq!(result, vec!["plugin-a", "plugin-b"]);

        let mut result2 = bus.subscribers_for("TimesheetStopped");
        result2.sort();
        assert_eq!(result2, vec!["plugin-a"]);
    }

    #[tokio::test]
    async fn publish_delivers_to_subscriber() {
        let bus = PluginEventBus::new(DEFAULT_BUS_CAPACITY, make_subs());
        let mut rx = bus.subscribe();

        let ev = DomainEvent::Plugin {
            plugin_id: "test".to_string(),
            aggregate: "leave_request".to_string(),
            event_type: "Submitted".to_string(),
            payload: serde_json::Value::Null,
        };
        let _ = bus.publish(ev);

        let received = rx.try_recv().expect("should receive event");
        assert_eq!(received.event_name(), "Submitted");
    }
}
