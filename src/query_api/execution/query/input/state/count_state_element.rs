// Corresponds to io.siddhi.query.api.execution.query.input.state.CountStateElement
use crate::query_api::siddhi_element::SiddhiElement;
use super::stream_state_element::StreamStateElement;

// Constant for ANY count, from Java's CountStateElement.ANY = -1
pub const ANY_COUNT: i32 = -1;

#[derive(Clone, Debug, PartialEq)] // Default is not straightforward
pub struct CountStateElement {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    // CountStateElement fields
    pub stream_state_element: Box<StreamStateElement>,
    pub min_count: i32, // Using i32 and ANY_COUNT to represent optionality, like Java
    pub max_count: i32, // Using i32 and ANY_COUNT
}

impl CountStateElement {
    pub fn new(stream_state_element: StreamStateElement, min_count: i32, max_count: i32) -> Self {
        CountStateElement {
            siddhi_element: SiddhiElement::default(),
            stream_state_element: Box::new(stream_state_element),
            min_count,
            max_count,
        }
    }
}

// No Default derive due to required StreamStateElement and specific count logic.
// A user would typically construct this with specific counts or ANY_COUNT.
// Defaulting min/max to ANY_COUNT might be an option if StreamStateElement was Default.
// impl Default for CountStateElement {
//     fn default() -> Self {
//         Self {
//             siddhi_element: SiddhiElement::default(),
//             stream_state_element: Box::default(), // Requires StreamStateElement::default()
//             min_count: ANY_COUNT,
//             max_count: ANY_COUNT,
//         }
//     }
// }
