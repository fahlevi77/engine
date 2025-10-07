// Corresponds to parts of io.eventflux.query.api.execution.query.output.ratelimit.EventOutputRate

// This struct holds the specific data for event-based rate limiting.
// The common EventFluxElement context will be in the main OutputRate struct.
// The OutputRateBehavior (All, First, Last) will also be associated in the OutputRate struct/enum.

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct EventsOutputRate {
    pub event_count: i32, // Corresponds to `value` in Java's EventOutputRate, defaults to 0
}

impl EventsOutputRate {
    pub fn new(event_count: i32) -> Self {
        EventsOutputRate { event_count }
    }
}
