// SPDX-License-Identifier: MIT OR Apache-2.0

use super::aggregation_runtime::AggregationRuntime;
use crate::core::event::value::AttributeValue;
use crate::query_api::aggregation::time_period::Duration as TimeDuration;

#[derive(Debug, Default)]
pub struct IncrementalDataAggregator;

impl IncrementalDataAggregator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn aggregate(
        runtime: &AggregationRuntime,
        duration: TimeDuration,
    ) -> Vec<Vec<AttributeValue>> {
        runtime.query_all(duration)
    }
}
