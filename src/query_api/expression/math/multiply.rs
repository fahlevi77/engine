// Corresponds to io.siddhi.query.api.expression.math.Multiply
use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Expression; // Main Expression enum

#[derive(Clone, Debug, PartialEq)] // Removed Default
pub struct Multiply {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    pub left_value: Box<Expression>,
    pub right_value: Box<Expression>,
}

impl Multiply {
    pub fn new(left_value: Expression, right_value: Expression) -> Self {
        Multiply {
            siddhi_element: SiddhiElement::default(),
            left_value: Box::new(left_value),
            right_value: Box::new(right_value),
        }
    }
}
