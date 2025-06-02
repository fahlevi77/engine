// Corresponds to io.siddhi.query.api.execution.query.input.state.AbsentStreamStateElement
use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::constant::Constant as ExpressionConstant; // Renamed
use super::stream_state_element::StreamStateElement; // For composition
use crate::query_api::execution::query::input::stream::BasicSingleInputStream;


#[derive(Clone, Debug, PartialEq)]
pub struct AbsentStreamStateElement {
    // Composes StreamStateElement for its fields + SiddhiElement context
    pub stream_state_element: StreamStateElement,

    // AbsentStreamStateElement specific fields
    pub waiting_time: Option<ExpressionConstant>, // Java's TimeConstant, which is a LongConstant
                                           // Making it Option as it might not always be present
}

impl AbsentStreamStateElement {
    pub fn new(basic_single_input_stream: BasicSingleInputStream, waiting_time: Option<ExpressionConstant>) -> Self {
        AbsentStreamStateElement {
            stream_state_element: StreamStateElement::new(basic_single_input_stream),
            waiting_time,
        }
    }

    pub fn get_basic_single_input_stream(&self) -> &BasicSingleInputStream {
        self.stream_state_element.get_basic_single_input_stream()
    }
}

// Delegate SiddhiElement implementation to the composed stream_state_element
impl SiddhiElement for AbsentStreamStateElement {
    fn query_context_start_index(&self) -> Option<(i32,i32)> { self.stream_state_element.query_context_start_index() }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) { self.stream_state_element.set_query_context_start_index(index); }
    fn query_context_end_index(&self) -> Option<(i32,i32)> { self.stream_state_element.query_context_end_index() }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) { self.stream_state_element.set_query_context_end_index(index); }
}
