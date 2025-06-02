// Corresponds to io.siddhi.query.api.execution.query.input.state.LogicalStateElement
use crate::query_api::siddhi_element::SiddhiElement;
// Changed from StreamStateElement to StateElement to allow combining, e.g., an absent stream and a regular stream.
use super::state_element::StateElement;

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LogicalStateElement {
    // SiddhiElement fields
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,

    // LogicalStateElement fields
    // These fields now take Box<StateElement> to allow different kinds of elements
    // (e.g. StreamStateElement or AbsentStreamStateElement) to be combined.
    // The StateElement should be a variant that represents a single effective stream.
    pub stream_state_element_1: Box<StateElement>,
    pub logical_type: Type,
    pub stream_state_element_2: Box<StateElement>,
}

impl LogicalStateElement {
    pub fn new(
        sse1: StateElement, // Changed from StreamStateElement
        logical_type: Type,
        sse2: StateElement, // Changed from StreamStateElement
    ) -> Self {
        // Add validation: sse1 and sse2 should be 'single stream' effective states
        // (e.g., StateElement::Stream, StateElement::AbsentStream).
        // They should not be complex states like Next, Every, Count, or another Logical.
        match sse1 {
            StateElement::Stream(_) | StateElement::AbsentStream(_) => {},
            _ => panic!("LogicalStateElement operand 1 must be a Stream or AbsentStream type StateElement"),
        }
        match sse2 {
            StateElement::Stream(_) | StateElement::AbsentStream(_) => {},
            _ => panic!("LogicalStateElement operand 2 must be a Stream or AbsentStream type StateElement"),
        }

        LogicalStateElement {
            query_context_start_index: None,
            query_context_end_index: None,
            stream_state_element_1: Box::new(sse1),
            logical_type,
            stream_state_element_2: Box::new(sse2),
        }
    }
}

impl SiddhiElement for LogicalStateElement {
    fn query_context_start_index(&self) -> Option<(i32,i32)> { self.query_context_start_index }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) { self.query_context_start_index = index; }
    fn query_context_end_index(&self) -> Option<(i32,i32)> { self.query_context_end_index }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) { self.query_context_end_index = index; }
}
