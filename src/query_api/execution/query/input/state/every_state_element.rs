// Corresponds to io.eventflux.query.api.execution.query.input.state.EveryStateElement
use super::state_element::StateElement;
use crate::query_api::eventflux_element::EventFluxElement; // Recursive definition
                                                           // Expression is not used here as per Java structure. 'within' is on StateInputStream.

#[derive(Clone, Debug, PartialEq)] // Default is not straightforward
pub struct EveryStateElement {
    pub eventflux_element: EventFluxElement, // Composed EventFluxElement

    // EveryStateElement fields
    pub state_element: Box<StateElement>,
    // The 'within' clause is associated with the whole pattern in StateInputStream,
    // not with individual 'every' elements in the Java API.
}

impl EveryStateElement {
    pub fn new(state_element: StateElement) -> Self {
        EveryStateElement {
            eventflux_element: EventFluxElement::default(),
            state_element: Box::new(state_element),
        }
    }
}

// No Default derive due to required Box<StateElement>.
