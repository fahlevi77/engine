use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::config::siddhi_query_context::SiddhiQueryContext;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::stream::stream_event::StreamEvent;
use crate::core::query::processor::{CommonProcessorMeta, ProcessingMode, Processor};
use crate::core::table::{InMemoryCompiledCondition, InMemoryCompiledUpdateSet, Table};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct UpdateTableProcessor {
    meta: CommonProcessorMeta,
    table: Arc<dyn Table>,
}

impl UpdateTableProcessor {
    pub fn new(
        table: Arc<dyn Table>,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Self {
        Self {
            meta: CommonProcessorMeta::new(app_ctx, query_ctx),
            table,
        }
    }
}

impl Processor for UpdateTableProcessor {
    fn process(&self, mut chunk: Option<Box<dyn ComplexEvent>>) {
        while let Some(mut event) = chunk {
            let next = event.set_next(None);
            if let Some(se) = event.as_any().downcast_ref::<StreamEvent>() {
                let old = se.before_window_data.clone();
                if let Some(new) = se.get_output_data() {
                    let cond = InMemoryCompiledCondition { values: old };
                    let us = InMemoryCompiledUpdateSet { values: new.to_vec() };
                    self.table.update(&cond, &us);
                }
            }
            chunk = next;
        }
    }

    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> {
        None
    }
    fn set_next_processor(&mut self, _next: Option<Arc<Mutex<dyn Processor>>>) {}
    fn clone_processor(&self, query_ctx: &Arc<SiddhiQueryContext>) -> Box<dyn Processor> {
        Box::new(Self::new(
            Arc::clone(&self.table),
            Arc::clone(&self.meta.siddhi_app_context),
            Arc::clone(query_ctx),
        ))
    }
    fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext> {
        Arc::clone(&self.meta.siddhi_app_context)
    }
    fn get_siddhi_query_context(&self) -> Arc<SiddhiQueryContext> {
        self.meta.get_siddhi_query_context()
    }
    fn get_processing_mode(&self) -> ProcessingMode {
        ProcessingMode::DEFAULT
    }
    fn is_stateful(&self) -> bool {
        true
    }
}
