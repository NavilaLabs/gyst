use async_trait::async_trait;
use eventually_projection::{Projector, RawEvent};

/// A composite projector built from dynamically registered sub-projectors.
///
/// Use this instead of (or alongside) `AdminProjector` / `TenantProjector`
/// when plugins need to register their own projectors at startup without
/// modifying core code.
///
/// ```rust,ignore
/// let mut projector = PluggableProjector::new();
/// projector.register(MyPluginProjector::new(pool.clone()));
/// // pass to ProjectionRunner as usual
/// ```
pub struct PluggableProjector {
    inner: Vec<Box<dyn Projector<Error = crate::Error> + Send>>,
}

impl Default for PluggableProjector {
    fn default() -> Self {
        Self::new()
    }
}

impl PluggableProjector {
    #[must_use]
    pub fn new() -> Self {
        Self { inner: vec![] }
    }

    pub fn register(&mut self, projector: impl Projector<Error = crate::Error> + 'static) {
        self.inner.push(Box::new(projector));
    }
}

#[async_trait]
impl Projector for PluggableProjector {
    type Error = crate::Error;

    async fn handle(&mut self, event: RawEvent) -> Result<(), Self::Error> {
        for projector in &mut self.inner {
            projector.handle(event.clone()).await?;
        }
        Ok(())
    }
}
