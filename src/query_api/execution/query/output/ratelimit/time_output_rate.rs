// Corresponds to parts of io.siddhi.query.api.execution.query.output.ratelimit.TimeOutputRate
use crate::query_api::expression::constant::Constant; // Using the unified Constant

// TimeOutputRate in Java has a Long `value` (milliseconds) and an `OutputRate.Type`.
// The `OutputRate.Type` (ALL, FIRST, LAST) will be part of the main OutputRate enum.
// This struct holds the time duration.

#[derive(Clone, Debug, PartialEq)]
pub struct TimeOutputRate {
    // In Java, this is a Long value representing milliseconds.
    // A TimeConstant in Siddhi stores its value as long.
    pub time_value_millis: i64,
}

impl TimeOutputRate {
    pub fn new(time_value_millis: i64) -> Self {
        TimeOutputRate { time_value_millis }
    }

    // Constructor from `Constant` if needed, assuming it's a Long or Time constant.
    // pub fn new_from_constant(constant: Constant) -> Result<Self, String> {
    //     match constant.value {
    //         crate::query_api::expression::constant::ConstantValue::Long(ms) |
    //         crate::query_api::expression::constant::ConstantValue::Time(ms) => Ok(Self::new(ms)),
    //         _ => Err("TimeOutputRate expects a Long or Time constant".to_string()),
    //     }
    // }
}
