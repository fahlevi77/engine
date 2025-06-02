use crate::query_api::siddhi_element::SiddhiElement;
use super::events_output_rate::EventsOutputRate;
use super::time_output_rate::TimeOutputRate;
use super::snapshot_output_rate::SnapshotOutputRate;
use crate::query_api::expression::constant::{Constant, ConstantValue};


// This enum corresponds to OutputRate.Type in Java (ALL, FIRST, LAST)
// SNAPSHOT type from Java's OutputRate.Type is specific to SnapshotOutputRate variant.
// EventOutputRate and TimeOutputRate can be ALL, FIRST, or LAST.
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum OutputRateBehavior {
    All,
    First,
    Last,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OutputRateVariant {
    // These variants represent the specific kind of rate limiting.
    // The OutputRateBehavior (All, First, Last) is associated with some of them.
    Events(EventsOutputRate, OutputRateBehavior),
    Time(TimeOutputRate, OutputRateBehavior),
    Snapshot(SnapshotOutputRate), // Snapshot implies its own behavior (typically all events within the snapshot)
}

#[derive(Clone, Debug, PartialEq)]
pub struct OutputRate {
    // SiddhiElement fields (OutputRate in Java is a SiddhiElement)
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,

    pub variant: OutputRateVariant,
}

impl OutputRate {
    // Constructors based on Java static factory methods in OutputRate.java

    // OutputRate.perEvents(Constant events) -> returns EventOutputRate
    // EventOutputRate.output(OutputRate.Type type) -> sets the behavior (ALL, FIRST, LAST)
    pub fn per_events(events: Constant, behavior: OutputRateBehavior) -> Result<Self, String> {
        let event_count = match events.value {
            ConstantValue::Long(val) => val as i32, // Java converts Long to int
            ConstantValue::Int(val) => val,
            _ => return Err("Unsupported constant type for events count, expected Int or Long.".to_string()),
        };
        Ok(OutputRate {
            query_context_start_index: None,
            query_context_end_index: None,
            variant: OutputRateVariant::Events(EventsOutputRate::new(event_count), behavior),
        })
    }

    // OutputRate.perTimePeriod(TimeConstant tc) / perTimePeriod(LongConstant lc) -> returns TimeOutputRate
    // TimeOutputRate.output(OutputRate.Type type) -> sets behavior
    pub fn per_time_period(time_constant: Constant, behavior: OutputRateBehavior) -> Result<Self, String> {
        let time_millis = match time_constant.value {
            ConstantValue::Time(ms) | ConstantValue::Long(ms) => ms,
            _ => return Err("Unsupported constant type for time period, expected Time or Long.".to_string()),
        };
        Ok(OutputRate {
            query_context_start_index: None,
            query_context_end_index: None,
            variant: OutputRateVariant::Time(TimeOutputRate::new(time_millis), behavior),
        })
    }

    // OutputRate.perSnapshot(TimeConstant tc) / perSnapshot(LongConstant lc) -> returns SnapshotOutputRate
    // SnapshotOutputRate in Java has a `type` field too, but it's usually fixed or defaults to OutputRate.Type.ALL
    // or implicitly SNAPSHOT. The prompt suggests `OutputRateType::SNAPSHOT`.
    pub fn per_snapshot(time_constant: Constant) -> Result<Self, String> {
        let time_millis = match time_constant.value {
            ConstantValue::Time(ms) | ConstantValue::Long(ms) => ms,
            _ => return Err("Unsupported constant type for snapshot period, expected Time or Long.".to_string()),
        };
        Ok(OutputRate {
            query_context_start_index: None,
            query_context_end_index: None,
            variant: OutputRateVariant::Snapshot(SnapshotOutputRate::new(time_millis)),
        })
    }

    // Helper method for Query's update_output_event_type
    pub fn is_snapshot(&self) -> bool {
        matches!(self.variant, OutputRateVariant::Snapshot(_))
    }
}

impl SiddhiElement for OutputRate {
    fn query_context_start_index(&self) -> Option<(i32,i32)> { self.query_context_start_index }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) { self.query_context_start_index = index; }
    fn query_context_end_index(&self) -> Option<(i32,i32)> { self.query_context_end_index }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) { self.query_context_end_index = index; }
}

// The OutputRateType enum from the prompt (ALL, FIRST, LAST, EVENTS, TIME, SNAPSHOT)
// seems to mix the behavior (ALL, FIRST, LAST) and the rate limiting method (EVENTS, TIME, SNAPSHOT).
// I've separated this into OutputRateBehavior and OutputRateVariant for clarity.
// If OutputRateType is strictly needed as defined in the prompt, it could be a field in OutputRate,
// and its value would be set based on the variant and behavior.
// For example:
// if variant is Events(_, All), type is EVENTS.
// if variant is Snapshot, type is SNAPSHOT.
// This seems redundant if the variant already captures this.
// The Java `OutputRate.Type` enum (ALL, FIRST, LAST, SNAPSHOT) is also a bit mixed.
// `SNAPSHOT` there seems to identify that it's a SnapshotOutputRate, while ALL, FIRST, LAST
// are behaviors for Event/Time output rates.
// My `OutputRateBehavior` maps to ALL, FIRST, LAST.
// My `OutputRateVariant` maps to EventOutputRate, TimeOutputRate, SnapshotOutputRate.
// The prompt's `OutputRateType` would be a combination/mapping from these.
// For now, I'm not including the exact `OutputRateType` enum from the prompt as its role is covered.
