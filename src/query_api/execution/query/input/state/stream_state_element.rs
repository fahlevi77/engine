// Corresponds to io.siddhi.query.api.execution.query.input.state.StreamStateElement
use crate::query_api::siddhi_element::SiddhiElement;
// BasicSingleInputStream functionality is now part of SingleInputStream via SingleInputStreamKind::Basic
use crate::query_api::execution::query::input::stream::SingleInputStream;

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct StreamStateElement {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    // StreamStateElement fields
    // Changed from BasicSingleInputStream to SingleInputStream.
    // This SingleInputStream instance would typically be of SingleInputStreamKind::Basic.
    pub basic_single_input_stream: SingleInputStream,
}

impl StreamStateElement {
    pub fn new(single_input_stream: SingleInputStream) -> Self { // Parameter type changed
        // It's up to the caller to ensure the provided SingleInputStream is appropriate
        // (e.g., of a Basic kind for most pattern stream elements).
        StreamStateElement {
            siddhi_element: SiddhiElement::default(),
            basic_single_input_stream: single_input_stream,
        }
    }

    pub fn get_single_input_stream(&self) -> &SingleInputStream { // Method name and return type changed
        &self.basic_single_input_stream
    }

    // Helper method for StateInputStream or other internal uses.
    pub(crate) fn get_stream_id(&self) -> &str {
        // SingleInputStream has get_stream_id_str() directly.
        self.basic_single_input_stream.get_stream_id_str()
    }
}

// SiddhiElement is composed. Access via self.siddhi_element.
// No direct impl of SiddhiElement trait needed if using composition and public field,
// unless it needs to be passed as `dyn SiddhiElement`.
// The StateElement enum's SiddhiElement impl handles dispatching to this composed element.
