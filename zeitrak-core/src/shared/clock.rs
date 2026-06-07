use chrono::{DateTime, Utc};

/// Abstracts the current wall-clock time so that it can be replaced in tests.
pub trait Clock: Send + Sync {
    /// Returns the current UTC time.
    fn now(&self) -> DateTime<Utc>;
}

/// Production implementation — delegates to [`Utc::now`].
#[derive(Debug, Clone, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
