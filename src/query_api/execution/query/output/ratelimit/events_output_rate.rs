// Corresponds to parts of io.siddhi.query.api.execution.query.output.ratelimit.EventOutputRate

// EventOutputRate in Java has an Integer `value` and an `OutputRate.Type` enum field.
// The `OutputRate.Type` (ALL, FIRST, LAST) will be part of the main OutputRate enum in output_rate.rs.
// This struct will only hold the event count.

#[derive(Clone, Debug, PartialEq)]
pub struct EventsOutputRate {
    pub event_count: i32, // Corresponds to `value` in Java's EventOutputRate
}

impl EventsOutputRate {
    pub fn new(event_count: i32) -> Self {
        EventsOutputRate { event_count }
    }
}
