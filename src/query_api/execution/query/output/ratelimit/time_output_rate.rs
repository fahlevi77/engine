// Corresponds to parts of io.eventflux.query.api.execution.query.output.ratelimit.TimeOutputRate

// This struct holds the specific data for time-based rate limiting.
// EventFluxElement context and OutputRateBehavior are in the main OutputRate struct.

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct TimeOutputRate {
    pub time_value_millis: i64, // Defaults to 0
}

impl TimeOutputRate {
    pub fn new(time_value_millis: i64) -> Self {
        TimeOutputRate { time_value_millis }
    }
}
