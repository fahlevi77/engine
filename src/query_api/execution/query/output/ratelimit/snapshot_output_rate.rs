// Corresponds to parts of io.eventflux.query.api.execution.query.output.ratelimit.SnapshotOutputRate

// This struct holds the specific data for snapshot-based rate limiting.
// EventFluxElement context is in the main OutputRate struct.
// Snapshot implies a specific behavior (usually OutputRateBehavior::All for the snapshot period).

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct SnapshotOutputRate {
    pub time_value_millis: i64, // Defaults to 0
}

impl SnapshotOutputRate {
    pub fn new(time_value_millis: i64) -> Self {
        SnapshotOutputRate { time_value_millis }
    }
}
