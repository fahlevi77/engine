// Corresponds to parts of io.siddhi.query.api.execution.query.output.ratelimit.SnapshotOutputRate
use crate::query_api::expression::constant::Constant; // Using the unified Constant

// SnapshotOutputRate in Java has a Long `value` (milliseconds) and its `type` is always SNAPSHOT.
// (It also inherits OutputRate.Type but it's fixed to SNAPSHOT implicitly by class,
// and explicitly set to ALL in the Java class, which is confusing. The prompt says SNAPSHOT type)
// This struct holds the time duration for the snapshot interval.

#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotOutputRate {
    // In Java, this is a Long value representing milliseconds.
    pub time_value_millis: i64,
}

impl SnapshotOutputRate {
    pub fn new(time_value_millis: i64) -> Self {
        SnapshotOutputRate { time_value_millis }
    }

    // Constructor from `Constant` if needed.
    // pub fn new_from_constant(constant: Constant) -> Result<Self, String> {
    //     match constant.value {
    //         crate::query_api::expression::constant::ConstantValue::Long(ms) |
    //         crate::query_api::expression::constant::ConstantValue::Time(ms) => Ok(Self::new(ms)),
    //         _ => Err("SnapshotOutputRate expects a Long or Time constant".to_string()),
    //     }
    // }
}
