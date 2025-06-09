use std::collections::HashMap;
use std::sync::Arc;

use crate::core::event::stream::stream_event::StreamEvent;
use crate::query_api::aggregation::time_period::Duration as TimeDuration;

use super::incremental_executor::IncrementalExecutor;

#[derive(Debug)]
pub struct AggregationRuntime {
    pub name: String,
    pub executors: HashMap<TimeDuration, Arc<IncrementalExecutor>>, // root executor by duration
}

impl AggregationRuntime {
    pub fn new(name: String, executors: HashMap<TimeDuration, Arc<IncrementalExecutor>>) -> Self {
        Self { name, executors }
    }

    pub fn process(&self, event: &StreamEvent) {
        // For now, always use the smallest duration executor
        if let Some(exec) = self.executors.values().next() {
            exec.execute(event);
        }
    }
}
