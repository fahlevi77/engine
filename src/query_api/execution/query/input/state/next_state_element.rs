// Corresponds to io.siddhi.query.api.execution.query.input.state.NextStateElement
use crate::query_api::siddhi_element::SiddhiElement;
use super::state_element::StateElement; // Recursive definition

#[derive(Clone, Debug, PartialEq)] // Default is not straightforward due to required Box<StateElement>
pub struct NextStateElement {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    // NextStateElement fields
    pub state_element: Box<StateElement>,
    pub next_state_element: Box<StateElement>, // Kept as required, not Option
}

impl NextStateElement {
    pub fn new(state_element: StateElement, next_state_element: StateElement) -> Self {
        NextStateElement {
            siddhi_element: SiddhiElement::default(),
            state_element: Box::new(state_element),
            next_state_element: Box::new(next_state_element),
        }
    }
}

// No Default derive or custom impl for now, as it requires a default StateElement.
// Similar to LogicalStateElement.
