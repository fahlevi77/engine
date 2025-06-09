use std::collections::HashMap;
use std::sync::Arc;

use crate::core::event::stream::stream_event::StreamEvent;
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::query_api::aggregation::time_period::Duration as TimeDuration;

use super::base_incremental_value_store::BaseIncrementalValueStore;

pub struct IncrementalExecutor {
    pub duration: TimeDuration,
    pub base_store: Arc<BaseIncrementalValueStore>,
    pub next: Option<Arc<IncrementalExecutor>>, // simplified chain
    pub group_by_fn: Box<dyn Fn(&StreamEvent) -> String + Send + Sync>,
}

impl std::fmt::Debug for IncrementalExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IncrementalExecutor")
            .field("duration", &self.duration)
            .finish()
    }
}

impl IncrementalExecutor {
    pub fn new(
        duration: TimeDuration,
        executors: Vec<Box<dyn ExpressionExecutor>>,
        group_by_fn: Box<dyn Fn(&StreamEvent) -> String + Send + Sync>,
    ) -> Self {
        Self {
            duration,
            base_store: Arc::new(BaseIncrementalValueStore::new(executors)),
            next: None,
            group_by_fn,
        }
    }

    pub fn execute(&self, event: &StreamEvent) {
        let key = (self.group_by_fn)(event);
        self.base_store.process(&key, event);
        if let Some(ref next) = self.next {
            for (_, ev) in self.base_store.get_grouped_events().into_iter() {
                next.execute(&ev);
            }
            self.base_store.clear();
        }
    }
}
