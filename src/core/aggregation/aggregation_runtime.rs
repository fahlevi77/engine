use std::collections::HashMap;
use std::sync::Arc;

use crate::core::event::stream::stream_event::StreamEvent;
use crate::query_api::aggregation::time_period::Duration as TimeDuration;

use super::incremental_executor::IncrementalExecutor;
use crate::core::table::{InMemoryTable, Table};

#[derive(Debug)]
pub struct AggregationRuntime {
    pub name: String,
    pub executors: HashMap<TimeDuration, Arc<IncrementalExecutor>>, // root executor by duration
    pub tables: HashMap<TimeDuration, Arc<InMemoryTable>>,          // aggregated data
}

impl AggregationRuntime {
    pub fn new(name: String, executors: HashMap<TimeDuration, Arc<IncrementalExecutor>>) -> Self {
        Self {
            name,
            executors,
            tables: HashMap::new(),
        }
    }

    pub fn process(&self, event: &StreamEvent) {
        // For now, always use the smallest duration executor
        if let Some(exec) = self.executors.values().next() {
            exec.execute(event);
        }
    }

    pub fn add_table(&mut self, dur: TimeDuration, table: Arc<InMemoryTable>) {
        self.tables.insert(dur, table);
    }

    pub fn get_table(&self, dur: TimeDuration) -> Option<Arc<InMemoryTable>> {
        self.tables.get(&dur).cloned()
    }

    pub fn query_all(
        &self,
        dur: TimeDuration,
    ) -> Vec<Vec<crate::core::event::value::AttributeValue>> {
        if let Some(table) = self.tables.get(&dur) {
            table.all_rows()
        } else {
            Vec::new()
        }
    }
}
