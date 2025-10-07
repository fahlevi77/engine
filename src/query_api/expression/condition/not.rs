// Corresponds to io.eventflux.query.api.expression.condition.Not
use crate::query_api::eventflux_element::EventFluxElement;
use crate::query_api::expression::Expression;

#[derive(Clone, Debug, PartialEq)] // Removed Default
pub struct Not {
    pub eventflux_element: EventFluxElement, // Composed EventFluxElement

    // Not specific field
    pub expression: Box<Expression>,
}

impl Not {
    pub fn new(expression: Expression) -> Self {
        Not {
            eventflux_element: EventFluxElement::default(),
            expression: Box::new(expression),
        }
    }
}
