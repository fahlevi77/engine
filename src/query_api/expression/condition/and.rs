// Corresponds to io.eventflux.query.api.expression.condition.And
use crate::query_api::eventflux_element::EventFluxElement;
use crate::query_api::expression::Expression;

#[derive(Clone, Debug, PartialEq)] // Removed Default
pub struct And {
    pub eventflux_element: EventFluxElement, // Composed EventFluxElement

    // And specific fields
    pub left_expression: Box<Expression>,
    pub right_expression: Box<Expression>,
}

impl And {
    pub fn new(left_expression: Expression, right_expression: Expression) -> Self {
        And {
            eventflux_element: EventFluxElement::default(),
            left_expression: Box::new(left_expression),
            right_expression: Box::new(right_expression),
        }
    }
}
