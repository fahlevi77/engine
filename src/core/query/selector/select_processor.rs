// siddhi_rust/src/core/query/selector/select_processor.rs
use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::config::siddhi_query_context::SiddhiQueryContext;
use crate::core::event::complex_event::{ComplexEvent, ComplexEventType};
use crate::core::event::stream::stream_event::StreamEvent;
use crate::core::event::value::AttributeValue;
use crate::core::query::processor::{Processor, CommonProcessorMeta, ProcessingMode};
use super::attribute::OutputAttributeProcessor;
use crate::query_api::definition::StreamDefinition as ApiStreamDefinition;

use std::sync::{Arc, Mutex};
use std::fmt::Debug;
use std::collections::VecDeque; // Using VecDeque for efficient chunk building

// Placeholders (assuming they are defined elsewhere or will be)
#[derive(Debug, Clone, Default)] pub struct GroupByKeyGeneratorPlaceholder {}
#[derive(Debug, Clone, Default)] pub struct OrderByEventComparatorPlaceholder {}
// OutputRateLimiter is the actual next processor for QuerySelector in Java
#[derive(Debug)] pub struct OutputRateLimiterPlaceholder { pub next_processor: Option<Arc<Mutex<dyn Processor>>>, pub siddhi_app_context: Arc<SiddhiAppContext> }
impl OutputRateLimiterPlaceholder {
    pub fn new(next_processor: Option<Arc<Mutex<dyn Processor>>>, siddhi_app_context: Arc<SiddhiAppContext>) -> Self { Self {next_processor, siddhi_app_context} }
    // TODO: Actual rate limiting logic would go into its process method
}
impl Processor for OutputRateLimiterPlaceholder {
    fn process(&self, complex_event_chunk: Option<Box<dyn ComplexEvent>>) { if let Some(ref next) = self.next_processor { next.lock().unwrap().process(complex_event_chunk); } }
    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> { self.next_processor.as_ref().map(Arc::clone) }
    fn set_next_processor(&mut self, next_processor: Option<Arc<Mutex<dyn Processor>>>) { self.next_processor = next_processor; }
    fn clone_processor(&self, siddhi_query_context: &Arc<SiddhiQueryContext>) -> Box<dyn Processor> {
        Box::new(Self::new(self.next_processor.as_ref().map(Arc::clone), Arc::clone(&siddhi_query_context.siddhi_app_context)))
    }
    fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext> { Arc::clone(&self.siddhi_app_context) }
    fn get_processing_mode(&self) -> ProcessingMode { ProcessingMode::DEFAULT }
    fn is_stateful(&self) -> bool { true } // Rate limiting is often stateful
}


/// A stream processor that handles SELECT clause projections.
#[derive(Debug)]
pub struct SelectProcessor {
    meta: CommonProcessorMeta,
    current_on: bool,
    expired_on: bool,
    contains_aggregator: bool,
    output_attribute_processors: Vec<OutputAttributeProcessor>,
    pub output_stream_definition: Arc<ApiStreamDefinition>,
    having_condition_executor: Option<Box<dyn crate::core::executor::expression_executor::ExpressionExecutor>>, // Changed placeholder
    is_group_by: bool,
    group_by_key_generator: Option<GroupByKeyGeneratorPlaceholder>,
    is_order_by: bool,
    order_by_event_comparator: Option<OrderByEventComparatorPlaceholder>,
    batching_enabled: bool,
    limit: Option<u64>,
    offset: Option<u64>,
}

impl SelectProcessor {
    pub fn new(
        api_selector: &crate::query_api::execution::query::selection::Selector,
        current_on: bool,
        expired_on: bool,
        siddhi_app_context: Arc<SiddhiAppContext>,
        siddhi_query_context: Arc<SiddhiQueryContext>,
        output_attribute_processors: Vec<OutputAttributeProcessor>,
        output_stream_definition: Arc<ApiStreamDefinition>,
        having_executor: Option<Box<dyn crate::core::executor::expression_executor::ExpressionExecutor>>,
        group_by_key_generator: Option<GroupByKeyGeneratorPlaceholder>,
        order_by_comparator: Option<OrderByEventComparatorPlaceholder>,
        batching_enabled: Option<bool>,
    ) -> Self {
        let query_name = siddhi_query_context.name.clone();
        let contains_aggregator_flag = output_attribute_processors
            .iter()
            .any(|oap| oap.is_aggregator());

        Self {
            meta: CommonProcessorMeta::new(siddhi_app_context, siddhi_query_context),
            current_on,
            expired_on,
            contains_aggregator: contains_aggregator_flag,
            output_attribute_processors,
            output_stream_definition,
            having_condition_executor: having_executor,
            is_group_by: group_by_key_generator.is_some(),
            group_by_key_generator,
            is_order_by: order_by_comparator.is_some(),
            order_by_event_comparator: order_by_comparator, // Corrected field init
            batching_enabled: batching_enabled.unwrap_or(true),
            limit: api_selector.limit.as_ref().and_then(|c| c.value.to_u64_for_limit_offset()),
            offset: api_selector.offset.as_ref().and_then(|c| c.value.to_u64_for_limit_offset()),
        }
    }
}

impl Processor for SelectProcessor {
    fn process(&self, complex_event_chunk: Option<Box<dyn ComplexEvent>>) {
        // Simplified process: iterate, transform, collect, then link and pass.
        // Does not handle batching, groupby, aggregation, having (fully), orderby, limit, offset.
        let mut input_event_opt = complex_event_chunk;
        let mut processed_events: Vec<Box<dyn ComplexEvent>> = Vec::new();

        while let Some(mut current_event_box) = input_event_opt {
            let next_event_in_original_chunk = current_event_box.set_next(None); // Detach

            let event_type = current_event_box.get_event_type();

            // Filter by event type (CURRENT/EXPIRED flags)
            let should_process_type = match event_type {
                ComplexEventType::Current if self.current_on => true,
                ComplexEventType::Expired if self.expired_on => true,
                ComplexEventType::Reset => true, // Resets often pass through for aggregators
                _ => false,
            };

            if !should_process_type && event_type != ComplexEventType::Reset {
                input_event_opt = next_event_in_original_chunk;
                continue;
            }

            // TODO: StateEventPopulater logic if applicable (skipped for now)

            // Apply attribute processors to generate output data
            let mut new_output_data = Vec::with_capacity(self.output_attribute_processors.len());
            for oap in &self.output_attribute_processors {
                new_output_data.push(oap.process(Some(current_event_box.as_ref())));
            }

            // Set the new output data on the event
            current_event_box.set_output_data(Some(new_output_data));
            // Preserve original event type unless it is a RESET
            if event_type != ComplexEventType::Reset {
                 current_event_box.set_event_type(event_type);
            }
            // Timestamp usually remains the same or is explicitly set by a projection.

            // Apply HAVING condition if present
            if let Some(ref having_exec) = self.having_condition_executor {
                let passes_having = match having_exec.execute(Some(current_event_box.as_ref())) {
                    Some(AttributeValue::Bool(true)) => true,
                    Some(AttributeValue::Bool(false)) | Some(AttributeValue::Null) | None => false,
                    _ => false,
                };
                if !passes_having {
                    input_event_opt = next_event_in_original_chunk;
                    continue;
                }
            }

            processed_events.push(current_event_box);
            input_event_opt = next_event_in_original_chunk;
        }

        // Reconstruct linked list from Vec (maintaining order)
        let mut new_chunk_head: Option<Box<dyn ComplexEvent>> = None;
        let mut tail_next_ref: &mut Option<Box<dyn ComplexEvent>> = &mut new_chunk_head;
        for event_box in processed_events {
            *tail_next_ref = Some(event_box);
            if let Some(ref mut current_tail) = *tail_next_ref {
                tail_next_ref = current_tail.mut_next_ref_option();
            }
        }

        if new_chunk_head.is_some() {
            if let Some(ref next_proc) = self.meta.next_processor {
                next_proc.lock().unwrap().process(new_chunk_head);
            }
        } else { // Pass on empty chunk if nothing resulted but there's a next processor
             if let Some(ref next_proc) = self.meta.next_processor {
                next_proc.lock().unwrap().process(None);
            }
        }
    }

    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> {
        self.meta.next_processor.as_ref().map(Arc::clone)
    }

    fn set_next_processor(&mut self, next_processor: Option<Arc<Mutex<dyn Processor>>>) {
        self.meta.next_processor = next_processor;
    }

    fn clone_processor(&self, siddhi_query_context: &Arc<SiddhiQueryContext>) -> Box<dyn Processor> {
        let cloned_oaps = self.output_attribute_processors.iter()
            .map(|oap| oap.clone_oap(&self.meta.siddhi_app_context))
            .collect();
        let cloned_having = self.having_condition_executor.as_ref()
            .map(|exec| exec.clone_executor(&self.meta.siddhi_app_context));

        Box::new(SelectProcessor {
            meta: CommonProcessorMeta::new(Arc::clone(&self.meta.siddhi_app_context), Arc::clone(siddhi_query_context)),
            current_on: self.current_on,
            expired_on: self.expired_on,
            contains_aggregator: self.contains_aggregator,
            output_attribute_processors: cloned_oaps,
            output_stream_definition: Arc::clone(&self.output_stream_definition),
            having_condition_executor: cloned_having,
            is_group_by: self.is_group_by,
            group_by_key_generator: self.group_by_key_generator.clone(),
            is_order_by: self.is_order_by,
            order_by_event_comparator: self.order_by_event_comparator.clone(),
            batching_enabled: self.batching_enabled,
            limit: self.limit,
            offset: self.offset,
        })
    }

    fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext> {
        Arc::clone(&self.meta.siddhi_app_context)
    }

    fn get_processing_mode(&self) -> ProcessingMode {
        if self.contains_aggregator
            || self.is_group_by
            || self.is_order_by
            || self.limit.is_some()
            || self.offset.is_some()
        {
            ProcessingMode::BATCH
        } else {
            ProcessingMode::DEFAULT
        }
    }

    fn is_stateful(&self) -> bool {
        self.contains_aggregator || self.is_group_by // Simplified
    }
}

// Helper on query_api::ConstantValueWithFloat for limit/offset
use crate::query_api::expression::constant::ConstantValueWithFloat as ApiConstantValue;
impl ApiConstantValue {
    fn to_u64_for_limit_offset(&self) -> Option<u64> {
        match self {
            ApiConstantValue::Int(i) if *i >= 0 => Some(*i as u64),
            ApiConstantValue::Long(l) if *l >= 0 => Some(*l as u64),
            _ => None, // Or error for invalid type/negative value
        }
    }
}
