// Corresponds to io.siddhi.query.api.expression.condition.Or
use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Expression;

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct Or {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    pub left_expression: Box<Expression>,
    pub right_expression: Box<Expression>,
}

impl Or {
    pub fn new(left_expression: Expression, right_expression: Expression) -> Self {
        Or {
            siddhi_element: SiddhiElement::default(),
            left_expression: Box::new(left_expression),
            right_expression: Box::new(right_expression),
        }
    }
}
