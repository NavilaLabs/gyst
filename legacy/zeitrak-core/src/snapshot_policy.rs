/// Declares how frequently the event store should write a snapshot for an aggregate.
///
/// The snapshot repository writes a snapshot after every save where
/// `aggregate_version % snapshot_every() == 0`.
///
/// # Default
///
/// The default `SNAPSHOT_EVERY` is `50`, matching `eventually_any`'s built-in default.
/// Override the constant for aggregates that accumulate events at very different rates.
///
/// Plugin-authored aggregates declare this value in their manifest (see §8.3 of the
/// plugin platform RFC) and the host injects it at plugin load time.
pub trait SnapshotPolicy {
    /// Number of events between automatic snapshot writes.
    ///
    /// A snapshot is written after any `save` where
    /// `new_aggregate_version % SNAPSHOT_EVERY == 0`.
    ///
    /// - `1`         — snapshot on every save (eager, minimises replay cost)
    /// - `50`        — snapshot every 50 events (default, balanced)
    /// - `u32::MAX`  — effectively disabled (always full event replay)
    const SNAPSHOT_EVERY: u32 = 50;
}
