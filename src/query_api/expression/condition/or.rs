// SPDX-License-Identifier: MIT OR Apache-2.0

// Corresponds to io.eventflux.query.api.expression.condition.Or
use crate::query_api::eventflux_element::EventFluxElement;
use crate::query_api::expression::Expression;

#[derive(Clone, Debug, PartialEq)] // Removed Default
pub struct Or {
    pub eventflux_element: EventFluxElement, // Composed EventFluxElement

    pub left_expression: Box<Expression>,
    pub right_expression: Box<Expression>,
}

impl Or {
    pub fn new(left_expression: Expression, right_expression: Expression) -> Self {
        Or {
            eventflux_element: EventFluxElement::default(),
            left_expression: Box::new(left_expression),
            right_expression: Box::new(right_expression),
        }
    }
}
