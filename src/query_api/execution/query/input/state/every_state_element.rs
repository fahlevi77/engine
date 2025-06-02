// Corresponds to io.siddhi.query.api.execution.query.input.state.EveryStateElement
use crate::query_api::siddhi_element::SiddhiElement;
use super::state_element::StateElement; // Recursive definition

#[derive(Clone, Debug, PartialEq)]
pub struct EveryStateElement {
    // SiddhiElement fields
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,

    // EveryStateElement fields
    pub state_element: Box<StateElement>,
}

impl EveryStateElement {
    pub fn new(state_element: StateElement) -> Self {
        EveryStateElement {
            query_context_start_index: None,
            query_context_end_index: None,
            state_element: Box::new(state_element),
        }
    }
}

impl SiddhiElement for EveryStateElement {
    fn query_context_start_index(&self) -> Option<(i32,i32)> { self.query_context_start_index }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) { self.query_context_start_index = index; }
    fn query_context_end_index(&self) -> Option<(i32,i32)> { self.query_context_end_index }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) { self.query_context_end_index = index; }
}
