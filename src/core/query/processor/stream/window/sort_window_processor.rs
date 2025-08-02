// siddhi_rust/src/core/query/processor/stream/window/sort_window_processor.rs
// Rust implementation of Siddhi SortWindowProcessor

use crate::core::config::{
    siddhi_app_context::SiddhiAppContext, siddhi_context::SiddhiContext,
    siddhi_query_context::SiddhiQueryContext,
};
use crate::core::event::complex_event::{ComplexEvent, ComplexEventType};
use crate::core::event::stream::StreamEvent;
use crate::core::query::processor::stream::window::WindowProcessor;
use crate::core::query::processor::{CommonProcessorMeta, ProcessingMode, Processor};
use crate::core::query::selector::order_by_event_comparator::OrderByEventComparator;
use crate::query_api::execution::query::input::handler::WindowHandler;
use crate::query_api::expression::{constant::ConstantValueWithFloat, Expression};

use std::sync::{Arc, Mutex};

/// A sort window maintains a fixed-size sliding window of events in sorted order
#[derive(Debug)]
pub struct SortWindowProcessor {
    /// Common processor metadata
    meta: CommonProcessorMeta,
    /// Maximum number of events to keep in the window
    length_to_keep: usize,
    /// Sorted buffer of events
    sorted_window: Arc<Mutex<Vec<Arc<StreamEvent>>>>,
    /// Comparator for sorting events
    comparator: OrderByEventComparator,
}

impl SortWindowProcessor {
    /// Create a new sort window processor
    pub fn new(
        length_to_keep: usize,
        comparator: OrderByEventComparator,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Self {
        SortWindowProcessor {
            meta: CommonProcessorMeta::new(app_ctx, query_ctx),
            length_to_keep,
            sorted_window: Arc::new(Mutex::new(Vec::new())),
            comparator,
        }
    }

    /// Create from window handler (standard factory pattern)
    pub fn from_handler(
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Self, String> {
        let params = handler.get_parameters();

        if params.is_empty() {
            return Err("Sort window requires at least a length parameter".to_string());
        }

        // First parameter: window length (required)
        let length_to_keep = match params.first() {
            Some(Expression::Constant(c)) => match &c.value {
                ConstantValueWithFloat::Int(i) => *i as usize,
                ConstantValueWithFloat::Long(l) => *l as usize,
                _ => return Err("Sort window length must be an integer".to_string()),
            },
            _ => return Err("Sort window length must be a constant".to_string()),
        };

        if length_to_keep <= 0 {
            return Err("Sort window length must be positive".to_string());
        }

        // For now, create a simple comparator that doesn't do complex expression parsing
        // TODO: Implement proper expression parsing for sort attributes
        let executors = Vec::new(); // Empty for now
        let ascending = vec![true]; // Default ascending
        let comparator = OrderByEventComparator::new(executors, ascending);

        Ok(Self::new(length_to_keep, comparator, app_ctx, query_ctx))
    }

    /// Process an incoming event
    fn process_event(&self, event: Arc<StreamEvent>) -> Result<Vec<Box<dyn ComplexEvent>>, String> {
        let mut sorted_buffer = self
            .sorted_window
            .lock()
            .map_err(|_| "Failed to acquire sort window lock".to_string())?;

        // Add the new event to the buffer
        sorted_buffer.push(Arc::clone(&event));

        let mut result = Vec::new();

        // Always emit the current event first
        let mut current_stream_event = event.as_ref().clone_without_next();
        current_stream_event.set_event_type(ComplexEventType::Current);
        result.push(Box::new(current_stream_event) as Box<dyn ComplexEvent>);

        // If we exceed the window size, we need to sort and remove the last element
        if sorted_buffer.len() > self.length_to_keep {
            // Sort the buffer - for now, just sort by timestamp as a simple default
            // TODO: Use proper attribute-based sorting when expression parsing is implemented
            sorted_buffer.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

            // Remove the last element (highest in sort order) and mark it as expired
            if let Some(expired_event) = sorted_buffer.pop() {
                let mut expired_stream_event = expired_event.as_ref().clone_without_next();
                expired_stream_event.set_event_type(ComplexEventType::Expired);
                result.push(Box::new(expired_stream_event) as Box<dyn ComplexEvent>);
            }
        }

        Ok(result)
    }
}

impl Processor for SortWindowProcessor {
    fn process(&self, complex_event_chunk: Option<Box<dyn ComplexEvent>>) {
        if let Some(ref next) = self.meta.next_processor {
            if let Some(chunk) = complex_event_chunk {
                let mut current_opt = Some(chunk.as_ref() as &dyn ComplexEvent);
                let mut all_events: Vec<Box<dyn ComplexEvent>> = Vec::new();

                while let Some(ev) = current_opt {
                    if let Some(se) = ev.as_any().downcast_ref::<StreamEvent>() {
                        match self.process_event(Arc::new(se.clone_without_next())) {
                            Ok(events) => {
                                all_events.extend(events);
                            }
                            Err(e) => {
                                eprintln!("Error processing sort window event: {}", e);
                            }
                        }
                    }
                    current_opt = ev.get_next();
                }

                // Send all events to next processor
                if !all_events.is_empty() {
                    let mut head: Option<Box<dyn ComplexEvent>> = None;
                    let mut tail = &mut head;

                    for event in all_events {
                        *tail = Some(event);
                        tail = tail.as_mut().unwrap().mut_next_ref_option();
                    }

                    next.lock().unwrap().process(head);
                }
            }
        }
    }

    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> {
        self.meta.next_processor.as_ref().map(Arc::clone)
    }

    fn set_next_processor(&mut self, next: Option<Arc<Mutex<dyn Processor>>>) {
        self.meta.next_processor = next;
    }

    fn clone_processor(&self, query_ctx: &Arc<SiddhiQueryContext>) -> Box<dyn Processor> {
        Box::new(Self::new(
            self.length_to_keep,
            // TODO: Clone comparator properly
            OrderByEventComparator::new(Vec::new(), Vec::new()),
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
        ProcessingMode::SLIDE
    }

    fn is_stateful(&self) -> bool {
        true
    }
}

impl WindowProcessor for SortWindowProcessor {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_window_creation() {
        // Basic test to ensure the structure can be created
        let executors = Vec::new();
        let ascending = vec![true];
        let comparator = OrderByEventComparator::new(executors, ascending);

        // This test mainly verifies compilation
        let siddhi_context = Arc::new(SiddhiContext::new());
        let app = Arc::new(crate::query_api::siddhi_app::SiddhiApp::new(
            "TestApp".to_string(),
        ));
        let app_ctx = Arc::new(SiddhiAppContext::new(
            siddhi_context,
            "TestApp".to_string(),
            app,
            String::new(),
        ));
        let query_ctx = Arc::new(SiddhiQueryContext::new(
            app_ctx.clone(),
            "test".to_string(),
            None,
        ));

        let _processor = SortWindowProcessor {
            meta: CommonProcessorMeta::new(app_ctx, query_ctx),
            length_to_keep: 3,
            sorted_window: Arc::new(Mutex::new(Vec::new())),
            comparator,
        };
    }
}
