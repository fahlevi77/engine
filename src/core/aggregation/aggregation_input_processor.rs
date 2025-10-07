use crate::core::config::{
    eventflux_app_context::EventFluxAppContext, eventflux_query_context::EventFluxQueryContext,
};
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::stream::stream_event::StreamEvent;
use crate::core::query::processor::{CommonProcessorMeta, ProcessingMode, Processor};
use std::sync::{Arc, Mutex};

use super::aggregation_runtime::AggregationRuntime;

#[derive(Debug)]
pub struct AggregationInputProcessor {
    meta: CommonProcessorMeta,
    runtime: Arc<Mutex<AggregationRuntime>>, // shared runtime
}

impl AggregationInputProcessor {
    pub fn new(
        runtime: Arc<Mutex<AggregationRuntime>>,
        app_ctx: Arc<EventFluxAppContext>,
        query_ctx: Arc<EventFluxQueryContext>,
    ) -> Self {
        Self {
            meta: CommonProcessorMeta::new(app_ctx, query_ctx),
            runtime,
        }
    }
}

impl Processor for AggregationInputProcessor {
    fn process(&self, mut chunk: Option<Box<dyn ComplexEvent>>) {
        while let Some(mut event) = chunk {
            let next = event.set_next(None);
            if let Some(se) = event.as_any().downcast_ref::<StreamEvent>() {
                self.runtime.lock().unwrap().process(se);
            }
            chunk = next;
        }
    }

    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> {
        None
    }
    fn set_next_processor(&mut self, _next: Option<Arc<Mutex<dyn Processor>>>) {}
    fn clone_processor(&self, qctx: &Arc<EventFluxQueryContext>) -> Box<dyn Processor> {
        Box::new(Self::new(
            Arc::clone(&self.runtime),
            Arc::clone(&self.meta.eventflux_app_context),
            Arc::clone(qctx),
        ))
    }
    fn get_eventflux_app_context(&self) -> Arc<EventFluxAppContext> {
        Arc::clone(&self.meta.eventflux_app_context)
    }
    fn get_eventflux_query_context(&self) -> Arc<EventFluxQueryContext> {
        self.meta.get_eventflux_query_context()
    }
    fn get_processing_mode(&self) -> ProcessingMode {
        ProcessingMode::DEFAULT
    }
    fn is_stateful(&self) -> bool {
        true
    }
}
