// Corresponds to io.siddhi.query.api.expression.math.Subtract
use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Expression; // Main Expression enum

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct Subtract {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    pub left_value: Box<Expression>,
    pub right_value: Box<Expression>,
}

impl Subtract {
    pub fn new(left_value: Expression, right_value: Expression) -> Self {
        Subtract {
            siddhi_element: SiddhiElement::default(),
            left_value: Box::new(left_value),
            right_value: Box::new(right_value),
        }
    }
}
