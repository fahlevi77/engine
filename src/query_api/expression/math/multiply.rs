// Corresponds to io.eventflux.query.api.expression.math.Multiply
use crate::query_api::eventflux_element::EventFluxElement;
use crate::query_api::expression::Expression; // Main Expression enum

#[derive(Clone, Debug, PartialEq)] // Removed Default
pub struct Multiply {
    pub eventflux_element: EventFluxElement, // Composed EventFluxElement

    pub left_value: Box<Expression>,
    pub right_value: Box<Expression>,
}

impl Multiply {
    pub fn new(left_value: Expression, right_value: Expression) -> Self {
        Multiply {
            eventflux_element: EventFluxElement::default(),
            left_value: Box::new(left_value),
            right_value: Box::new(right_value),
        }
    }
}
