// Corresponds to io.siddhi.query.api.execution.query.input.state.CountStateElement
use crate::query_api::siddhi_element::SiddhiElement;
use super::stream_state_element::StreamStateElement;

// Constant for ANY count, from Java's CountStateElement.ANY = -1
pub const ANY_COUNT: i32 = -1;

#[derive(Clone, Debug, PartialEq)]
pub struct CountStateElement {
    // SiddhiElement fields
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,

    // CountStateElement fields
    pub stream_state_element: Box<StreamStateElement>, // Contains BasicSingleInputStream
    pub min_count: i32,
    pub max_count: i32,
}

impl CountStateElement {
    pub fn new(stream_state_element: StreamStateElement, min_count: i32, max_count: i32) -> Self {
        CountStateElement {
            query_context_start_index: None,
            query_context_end_index: None,
            stream_state_element: Box::new(stream_state_element),
            min_count,
            max_count,
        }
    }
}

impl SiddhiElement for CountStateElement {
    fn query_context_start_index(&self) -> Option<(i32,i32)> { self.query_context_start_index }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) { self.query_context_start_index = index; }
    fn query_context_end_index(&self) -> Option<(i32,i32)> { self.query_context_end_index }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) { self.query_context_end_index = index; }
}
