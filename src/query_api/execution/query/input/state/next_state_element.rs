// Corresponds to io.siddhi.query.api.execution.query.input.state.NextStateElement
use crate::query_api::siddhi_element::SiddhiElement;
use super::state_element::StateElement; // Recursive definition

#[derive(Clone, Debug, PartialEq)]
pub struct NextStateElement {
    // SiddhiElement fields
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,

    // NextStateElement fields
    pub state_element: Box<StateElement>,
    pub next_state_element: Box<StateElement>,
}

impl NextStateElement {
    pub fn new(state_element: StateElement, next_state_element: StateElement) -> Self {
        NextStateElement {
            query_context_start_index: None,
            query_context_end_index: None,
            state_element: Box::new(state_element),
            next_state_element: Box::new(next_state_element),
        }
    }
}

impl SiddhiElement for NextStateElement {
    fn query_context_start_index(&self) -> Option<(i32,i32)> { self.query_context_start_index }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) { self.query_context_start_index = index; }
    fn query_context_end_index(&self) -> Option<(i32,i32)> { self.query_context_end_index }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) { self.query_context_end_index = index; }
}
