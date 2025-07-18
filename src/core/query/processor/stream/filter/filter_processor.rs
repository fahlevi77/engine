// siddhi_rust/src/core/query/processor/stream/filter/filter_processor.rs
use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::config::siddhi_query_context::SiddhiQueryContext; // For clone_processor
use crate::core::event::complex_event::ComplexEvent; // Trait
use crate::core::event::value::AttributeValue;
use crate::core::executor::expression_executor::ExpressionExecutor; // Trait
use crate::core::query::processor::{CommonProcessorMeta, ProcessingMode, Processor}; // Use CommonProcessorMeta
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

// FilterProcessor doesn't exist as a distinct class in Java Siddhi's core structure.
// Filtering is typically part of SingleStreamProcessor or JoinProcessor using a ConditionExpressionExecutor.
// This Rust struct is created as per the prompt to represent a dedicated filter.
/// A stream processor that filters events based on a condition.
#[derive(Debug)]
pub struct FilterProcessor {
    meta: CommonProcessorMeta, // Common fields like siddhi_app_context, query_name, next_processor
    condition_executor: Box<dyn ExpressionExecutor>,
    // is_per_event_trace_enabled: bool, // Can get from siddhi_app_context.get_siddhi_context().get_statistics_configuration()
}

impl FilterProcessor {
    pub fn new(
        condition_executor: Box<dyn ExpressionExecutor>,
        siddhi_app_context: Arc<SiddhiAppContext>,
        siddhi_query_context: Arc<SiddhiQueryContext>, // query_name is in here
    ) -> Result<Self, String> {
        if condition_executor.get_return_type() != crate::query_api::definition::AttributeType::BOOL
        {
            return Err(format!(
                "Filter condition executor must return BOOL, but found {:?}",
                condition_executor.get_return_type()
            ));
        }
        Ok(Self {
            meta: CommonProcessorMeta::new(siddhi_app_context, siddhi_query_context),
            condition_executor,
        })
    }
}

impl Processor for FilterProcessor {
    fn process(&self, mut complex_event_chunk: Option<Box<dyn ComplexEvent>>) {
        // This simplified version processes events one by one from the input chunk
        // and builds a new chunk for the filtered events.
        // More advanced/performant versions might try to modify the chunk in-place
        // or use an event pool.

        let mut filtered_events: Vec<Box<dyn ComplexEvent>> = Vec::new();

        let mut current_event_opt = complex_event_chunk;
        while let Some(mut current_event_box) = current_event_opt {
            // Detach the current event from the original chunk to process it individually.
            let next_event_in_original_chunk = current_event_box.set_next(None);

            let passes_filter = match self
                .condition_executor
                .execute(Some(current_event_box.as_ref()))
            {
                Some(AttributeValue::Bool(true)) => true,
                Some(AttributeValue::Bool(false)) | Some(AttributeValue::Null) => false,
                None => false, // Error or no value from condition executor, filter out
                _ => {
                    // log_error!("Filter condition did not return a boolean for event: {:?}", current_event_box);
                    false
                }
            };

            if passes_filter {
                filtered_events.push(current_event_box);
            } // else: event is dropped

            current_event_opt = next_event_in_original_chunk;
        }

        // Reconstruct linked list from Vec of passed events
        let mut filtered_chunk_head: Option<Box<dyn ComplexEvent>> = None;
        let mut tail_next_ref: &mut Option<Box<dyn ComplexEvent>> = &mut filtered_chunk_head;
        for mut event_box in filtered_events {
            *tail_next_ref = Some(event_box);
            if let Some(ref mut current_tail) = *tail_next_ref {
                tail_next_ref = current_tail.mut_next_ref_option();
            }
        }

        if let Some(ref next_proc_arc) = self.meta.next_processor {
            next_proc_arc.lock().unwrap().process(filtered_chunk_head);
        }
    }

    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> {
        self.meta.next_processor.as_ref().map(Arc::clone)
    }

    fn set_next_processor(&mut self, next_processor: Option<Arc<Mutex<dyn Processor>>>) {
        self.meta.next_processor = next_processor;
    }

    fn clone_processor(
        &self,
        siddhi_query_context: &Arc<SiddhiQueryContext>,
    ) -> Box<dyn Processor> {
        let cloned_condition_executor = self
            .condition_executor
            .clone_executor(&siddhi_query_context.siddhi_app_context);
        Box::new(FilterProcessor {
            meta: CommonProcessorMeta::new(
                Arc::clone(&siddhi_query_context.siddhi_app_context),
                Arc::clone(siddhi_query_context),
            ),
            condition_executor: cloned_condition_executor,
        })
    }

    fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext> {
        Arc::clone(&self.meta.siddhi_app_context)
    }

    fn get_siddhi_query_context(&self) -> Arc<SiddhiQueryContext> {
        self.meta.get_siddhi_query_context()
    }

    fn get_processing_mode(&self) -> ProcessingMode {
        ProcessingMode::DEFAULT // Filter is usually a pass-through or simple mode
    }

    fn is_stateful(&self) -> bool {
        false // FilterProcessor is typically stateless
    }
}
