//! Plugin audit sink (Phase H — §14, step 33).
//!
//! [`PluginAuditSink`] implements `dioxus-extism`'s [`CrossPluginAuditSink`]
//! trait and writes every cross-plugin call event to the `plugin_audit` table
//! in the admin database.
//!
//! Because [`CrossPluginAuditSink::record`] is synchronous and must not block,
//! all writes are forwarded over an unbounded channel to a background Tokio task
//! that performs the actual `INSERT`. The channel is effectively fire-and-forget:
//! a full channel causes the event to be silently dropped rather than blocking
//! or panicking the caller.

use std::time::SystemTime;

use dioxus_extism_host::{CallOutcome, CrossPluginAuditSink, CrossPluginCallEvent};
use tokio::sync::mpsc;
use zeitrak_infrastructure_impl::ConnectedAdminPool;

/// A record queued for persistence.
struct AuditRow {
    occurred_at: String,
    caller_plugin_id: String,
    target_plugin_id: String,
    function_name: String,
    outcome: &'static str,
    error_message: Option<String>,
    duration_ms: Option<i64>,
}

/// [`CrossPluginAuditSink`] implementation that persists events to the
/// `plugin_audit` table in the admin database.
///
/// Construct with [`PluginAuditSink::new`], then:
///
/// 1. Register the sink with `PluginRuntimeBuilder::with_audit_sink`.
/// 2. `tokio::spawn` the drain future returned by `new`.
pub struct PluginAuditSink {
    tx: mpsc::UnboundedSender<AuditRow>,
}

impl PluginAuditSink {
    /// Build the sink and a background drain that writes rows to the admin DB.
    ///
    /// The returned future must be spawned (e.g. `tokio::spawn(drain)`) before
    /// any events are recorded; otherwise the internal channel fills and events
    /// are silently discarded.
    pub fn new(
        pool: ConnectedAdminPool,
    ) -> (Self, impl std::future::Future<Output = ()> + Send + 'static) {
        let (tx, mut rx) = mpsc::unbounded_channel::<AuditRow>();

        let drain = async move {
            let pool_ref: &sqlx::AnyPool = pool.as_ref();
            while let Some(row) = rx.recv().await {
                let _ = sqlx::query(
                    "INSERT INTO plugin_audit \
                     (occurred_at, plugin_id, function_name, trust_tier, outcome, \
                      error_message, duration_ms) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7)",
                )
                .bind(&row.occurred_at)
                // Record the caller as the primary plugin_id for the audit row;
                // the target is appended to function_name for full traceability.
                .bind(format!(
                    "{} → {}",
                    row.caller_plugin_id, row.target_plugin_id
                ))
                .bind(&row.function_name)
                .bind("cross-plugin") // trust_tier is unknown at this call site
                .bind(row.outcome)
                .bind(&row.error_message)
                .bind(row.duration_ms)
                .execute(pool_ref)
                .await;
            }
        };

        (Self { tx }, drain)
    }
}

impl CrossPluginAuditSink for PluginAuditSink {
    fn record(&self, event: CrossPluginCallEvent) {
        let occurred_at = system_time_to_rfc3339(event.timestamp);
        let (outcome, error_message, duration_ms) = match &event.outcome {
            CallOutcome::Allowed { duration } => {
                // Truncating to i64 milliseconds is safe: i64::MAX ms ≈ 292 million years.
                #[allow(clippy::cast_possible_truncation)]
                let ms = duration.as_millis() as i64;
                ("allowed", None, Some(ms))
            }
            CallOutcome::Denied { reason } => ("denied", Some(format!("{reason:?}")), None),
            CallOutcome::Failed { error_kind } => ("failed", Some(format!("{error_kind:?}")), None),
            // Non-exhaustive: forward-compatible with future dioxus-extism variants.
            _ => ("unknown", None, None),
        };

        let row = AuditRow {
            occurred_at,
            caller_plugin_id: event.caller.0.clone(),
            target_plugin_id: event.target.0.clone(),
            function_name: event.function,
            outcome,
            error_message,
            duration_ms,
        };

        // Ignore send errors: if the receiver is gone, events are dropped.
        let _ = self.tx.send(row);
    }
}

fn system_time_to_rfc3339(time: SystemTime) -> String {
    use std::time::UNIX_EPOCH;
    let total_secs = time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Minimal ISO 8601 UTC string: YYYY-MM-DDTHH:MM:SSZ
    let sec = total_secs % 60;
    let min = (total_secs / 60) % 60;
    let hour = (total_secs / 3600) % 24;
    let days = total_secs / 86_400;
    // Days since 1970-01-01 → Gregorian date (simple algorithm, valid through ~year 2100)
    let adj = days + 719_468;
    let era = adj / 146_097;
    let doe = adj - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}
