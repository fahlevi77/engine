// Corresponds to io.siddhi.query.api.expression.condition.And
use crate::query_api::expression::Expression;
use crate::query_api::siddhi_element::SiddhiElement;

#[derive(Clone, Debug, PartialEq)] // Removed Default
pub struct And {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    // And specific fields
    pub left_expression: Box<Expression>,
    pub right_expression: Box<Expression>,
}

impl And {
    pub fn new(left_expression: Expression, right_expression: Expression) -> Self {
        And {
            siddhi_element: SiddhiElement::default(),
            left_expression: Box::new(left_expression),
            right_expression: Box::new(right_expression),
        }
    }
}
