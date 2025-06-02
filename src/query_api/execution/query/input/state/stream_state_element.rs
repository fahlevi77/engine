// Corresponds to io.siddhi.query.api.execution.query.input.state.StreamStateElement
use crate::query_api::siddhi_element::SiddhiElement;
// BasicSingleInputStream is from the stream module
use crate::query_api::execution::query::input::stream::BasicSingleInputStream;

#[derive(Clone, Debug, PartialEq)]
pub struct StreamStateElement {
    // SiddhiElement fields
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,

    // StreamStateElement fields
    pub basic_single_input_stream: BasicSingleInputStream,
    // In Java, this is `private final BasicSingleInputStream basicSingleInputStream;`
    // Making it public here for direct access if needed, or provide getter.
}

impl StreamStateElement {
    pub fn new(basic_single_input_stream: BasicSingleInputStream) -> Self {
        StreamStateElement {
            query_context_start_index: None,
            query_context_end_index: None,
            basic_single_input_stream,
        }
    }

    pub fn get_basic_single_input_stream(&self) -> &BasicSingleInputStream {
        &self.basic_single_input_stream
    }
}

impl SiddhiElement for StreamStateElement {
    fn query_context_start_index(&self) -> Option<(i32,i32)> { self.query_context_start_index }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) { self.query_context_start_index = index; }
    fn query_context_end_index(&self) -> Option<(i32,i32)> { self.query_context_end_index }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) { self.query_context_end_index = index; }
}
