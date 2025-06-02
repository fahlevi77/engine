// Corresponds to io.siddhi.query.api.execution.query.input.state.AbsentStreamStateElement
use crate::query_api::siddhi_element::SiddhiElement; // For direct composition if not delegating
use crate::query_api::expression::constant::Constant as ExpressionConstant;
use super::stream_state_element::StreamStateElement;
use crate::query_api::execution::query::input::stream::BasicSingleInputStream;


#[derive(Clone, Debug, PartialEq)] // Default is not straightforward
pub struct AbsentStreamStateElement {
    // In Java, AbsentStreamStateElement extends StreamStateElement.
    // So, it should compose StreamStateElement, which in turn composes SiddhiElement.
    // The prompt suggests direct composition of siddhi_element, but that would lose
    // the basic_single_input_stream from StreamStateElement if not careful.
    // Sticking to composing StreamStateElement.
    pub stream_state_element: StreamStateElement,

    // AbsentStreamStateElement specific fields
    pub waiting_time: Option<ExpressionConstant>,
}

impl AbsentStreamStateElement {
    // Constructor takes BasicSingleInputStream to create the inner StreamStateElement,
    // and the waiting_time.
    pub fn new(basic_single_input_stream: BasicSingleInputStream, waiting_time: Option<ExpressionConstant>) -> Self {
        AbsentStreamStateElement {
            stream_state_element: StreamStateElement::new(basic_single_input_stream),
            waiting_time,
        }
    }

    // Constructor that takes a pre-constructed StreamStateElement, as per prompt's general direction.
    // This is more flexible if StreamStateElement is already formed.
    pub fn new_with_stream_state(stream_state_element: StreamStateElement, waiting_time: Option<ExpressionConstant>) -> Self {
        AbsentStreamStateElement {
            stream_state_element,
            waiting_time,
        }
    }

    pub fn get_basic_single_input_stream(&self) -> &BasicSingleInputStream {
        self.stream_state_element.get_basic_single_input_stream()
    }

    // Expose siddhi_element for direct access if needed by StateElement enum dispatch.
    // This provides access to the SiddhiElement composed within the inner StreamStateElement.
    pub fn siddhi_element(&self) -> &SiddhiElement {
        &self.stream_state_element.siddhi_element
    }
    pub fn siddhi_element_mut(&mut self) -> &mut SiddhiElement {
        &mut self.stream_state_element.siddhi_element
    }
}

// No Default derive due to required StreamStateElement.

// No direct SiddhiElement impl for AbsentStreamStateElement.
// The StateElement enum will access the siddhi_element from the composed stream_state_element.
// If AbsentStreamStateElement needed to be passed as `dyn SiddhiElement`, it would need:
// impl SiddhiElement for AbsentStreamStateElement {
//     fn query_context_start_index(&self) -> Option<(i32,i32)> { self.stream_state_element.siddhi_element.query_context_start_index }
//     // ... and so on for other SiddhiElement methods, delegating to self.stream_state_element.siddhi_element
// }
// This is effectively what StateElement enum's SiddhiElement impl will do via the siddhi_element() helper.
