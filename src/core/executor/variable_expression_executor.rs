// siddhi_rust/src/core/executor/variable_expression_executor.rs
// Corresponds to io.siddhi.core.executor.VariableExpressionExecutor
use super::expression_executor::ExpressionExecutor;
use crate::core::event::complex_event::ComplexEvent; // Trait
use crate::core::event::value::AttributeValue;
use crate::core::event::stream::StreamEvent; // For accessing specific data arrays if event is StreamEvent
use crate::core::event::state::StateEvent;   // For accessing specific data arrays if event is StateEvent
use crate::query_api::definition::Attribute as QueryApiAttribute; // For Attribute::Type and the Attribute definition itself
// use crate::query_api::expression::Variable as QueryApiVariable; // Original variable definition (not strictly needed for execution)
use crate::core::config::siddhi_app_context::SiddhiAppContext; // May be needed for dynamic resolution
use std::sync::Arc;

// In Java, VariableExpressionExecutor has an int[] position.
// position[0]: stream event chain index (for StateEvent, which stream in the pattern)
// position[1]: stream event index in chain (0 for current, -1 for last, -2 for second last etc., or positive index)
// position[2]: stream attribute type index (0: beforeWindowData, 1: onAfterWindowData, 2: outputData for StreamEvent;
//                                          or 3: STATE_OUTPUT_DATA_INDEX for StateEvent's own outputData)
// position[3]: stream attribute index in the selected array

// We'll simplify this for Rust. Position can be a more structured enum or struct.
// For now, let's assume position means:
// (stream_array_index_for_state_event, data_array_selector, attribute_index_in_array)
// where data_array_selector indicates which Vec<AttributeValue> to pick from StreamEvent/StateEvent.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventDataArrayType {
    BeforeWindowData,   // for StreamEvent
    OnAfterWindowData,  // for StreamEvent
    OutputData,         // for StreamEvent's direct output or StateEvent's direct output
}

#[derive(Debug, Clone)]
pub struct VariablePosition {
    pub stream_event_chain_index: Option<usize>, // For StateEvent: which StreamEvent in its array
                                             // For StreamEvent: None or 0
    pub stream_event_index_in_chain: i32, // 0 for current, -1 for last, etc. (from SiddhiConstants)
                                          // TODO: Use constants from crate::query_api::constants if defined
    pub array_type: EventDataArrayType, // Which data array to access
    pub attribute_index: usize,         // Index within that array
}


#[derive(Debug, Clone)]
pub struct VariableExpressionExecutor {
    // attribute_definition: QueryApiAttribute, // The definition of the attribute being accessed
    position: VariablePosition, // Describes how to locate the attribute in a ComplexEvent
    return_type: QueryApiAttribute::Type,

    // For more complex scenarios like accessing attributes from event tables or stores dynamically.
    // siddhi_app_context: Option<Arc<SiddhiAppContext>>,
    // attribute_dynamic_resolve_type: AttributeDynamicResolveType, // Placeholder from prompt
}

impl VariableExpressionExecutor {
    pub fn new(
        // attribute_definition: QueryApiAttribute,
        position: VariablePosition,
        return_type: QueryApiAttribute::Type,
        // siddhi_app_context: Option<Arc<SiddhiAppContext>>,
    ) -> Self {
        Self {
            // attribute_definition,
            position,
            return_type,
            // siddhi_app_context,
        }
    }
}

impl ExpressionExecutor for VariableExpressionExecutor {
    fn execute(&self, event_opt: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let event = event_opt?;

        // Try to downcast to StateEvent first, as it's more general for patterns/joins
        if let Some(state_event) = event.as_any().downcast_ref::<StateEvent>() {
            let stream_idx = self.position.stream_event_chain_index.unwrap_or(0); // Default to first stream if not specified

            // Logic to get the correct StreamEvent from StateEvent's array based on stream_event_index_in_chain
            let target_stream_event: Option<&StreamEvent> = {
                if let Some(chain_head) = state_event.stream_events.get(stream_idx).and_then(|opt_se| opt_se.as_ref()) {
                    // TODO: Implement full logic for positive index, CURRENT (-1), LAST (-2), etc.
                    // This is complex as StreamEvent.next is Box<dyn ComplexEvent>.
                    // For now, just access the head of the chain at stream_idx if index_in_chain is 0.
                    if self.position.stream_event_index_in_chain == 0 { // Simplified: only current event in chain
                        Some(chain_head)
                    } else {
                        // log_warn!("Accessing specific event in chain (e.g., event[{}]) not fully implemented for StateEvent", self.position.stream_event_index_in_chain);
                        None
                    }
                } else { None }
            };

            if let Some(se) = target_stream_event {
                match self.position.array_type {
                    EventDataArrayType::BeforeWindowData => se.before_window_data.get(self.position.attribute_index).cloned(),
                    EventDataArrayType::OnAfterWindowData => se.on_after_window_data.get(self.position.attribute_index).cloned(),
                    EventDataArrayType::OutputData => se.output_data.as_ref().and_then(|od| od.get(self.position.attribute_index).cloned()),
                }
            } else {
                // If it's StateEvent's own output data (position[2] == STATE_OUTPUT_DATA_INDEX in Java)
                if self.position.array_type == EventDataArrayType::OutputData && self.position.stream_event_chain_index.is_none() {
                     state_event.output_data.as_ref().and_then(|od| od.get(self.position.attribute_index).cloned())
                } else {
                    None
                }
            }

        } else if let Some(stream_event) = event.as_any().downcast_ref::<StreamEvent>() {
            // If it's a simple StreamEvent (not part of a StateEvent)
            // stream_event_chain_index and stream_event_index_in_chain usually don't apply or are 0.
            match self.position.array_type {
                EventDataArrayType::BeforeWindowData => stream_event.before_window_data.get(self.position.attribute_index).cloned(),
                EventDataArrayType::OnAfterWindowData => stream_event.on_after_window_data.get(self.position.attribute_index).cloned(),
                EventDataArrayType::OutputData => stream_event.output_data.as_ref().and_then(|od| od.get(self.position.attribute_index).cloned()),
            }
        } else {
            // Event is some other ComplexEvent implementation
            // log_warn!("VariableExpressionExecutor received an unknown ComplexEvent type.");
            None
        }
    }

    fn get_return_type(&self) -> QueryApiAttribute::Type {
        self.return_type
    }

    // fn clone_executor(&self) -> Box<dyn ExpressionExecutor> {
    //     Box::new(self.clone())
    // }
}

// The AttributeDynamicResolveType enum from the prompt might be used if the variable
// needs to be resolved from a table or store dynamically during execution.
// This would require siddhi_app_context and table access logic.
// For now, VariableExpressionExecutor assumes direct data access from the event.
