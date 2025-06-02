// Corresponds to io.siddhi.query.api.execution.query.input.state.EveryStateElement
use crate::query_api::siddhi_element::SiddhiElement;
use super::state_element::StateElement; // Recursive definition
// Expression is not used here as per Java structure. 'within' is on StateInputStream.

#[derive(Clone, Debug, PartialEq)] // Default is not straightforward
pub struct EveryStateElement {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    // EveryStateElement fields
    pub state_element: Box<StateElement>,
    // The 'within' clause is associated with the whole pattern in StateInputStream,
    // not with individual 'every' elements in the Java API.
}

impl EveryStateElement {
    pub fn new(state_element: StateElement) -> Self {
        EveryStateElement {
            siddhi_element: SiddhiElement::default(),
            state_element: Box::new(state_element),
        }
    }
}

// No Default derive due to required Box<StateElement>.
