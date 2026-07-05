use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// A serialised snapshot of a domain event, decoupled from its aggregate type.
///
/// Plugins that need to react to domain events (webhooks, audit logs,
/// notifications) subscribe via [`DomainEventHandler`] and receive these
/// envelopes without needing to know the concrete event type.
#[derive(Debug, Clone)]
pub struct DomainEventEnvelope {
    pub aggregate_type: &'static str,
    pub aggregate_id: String,
    pub event_name: &'static str,
    pub payload: serde_json::Value,
    pub occurred_at: DateTime<Utc>,
}

/// A single observer that reacts to published domain events.
///
/// Register implementations with [`EventBus::register`] at startup.
#[async_trait]
pub trait DomainEventHandler: Send + Sync {
    /// Called after an aggregate is successfully saved.
    ///
    /// # Errors
    ///
    /// Returns an error if the handler cannot process the event.
    async fn on_event(
        &self,
        event: &DomainEventEnvelope,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Fan-out bus that delivers each event to every registered handler in order.
#[derive(Default)]
pub struct EventBus {
    handlers: Vec<Box<dyn DomainEventHandler>>,
}

impl EventBus {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, handler: impl DomainEventHandler + 'static) {
        self.handlers.push(Box::new(handler));
    }

    /// Publishes `envelope` to every registered handler.
    ///
    /// Handlers are called sequentially; the first error aborts the chain.
    ///
    /// # Errors
    ///
    /// Returns the first error produced by any registered handler.
    pub async fn publish(
        &self,
        envelope: DomainEventEnvelope,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for handler in &self.handlers {
            handler.on_event(&envelope).await?;
        }
        Ok(())
    }
}
