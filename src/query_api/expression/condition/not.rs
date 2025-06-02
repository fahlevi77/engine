// Corresponds to io.siddhi.query.api.expression.condition.Not
use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Expression;

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct Not {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    // Not specific field
    pub expression: Box<Expression>,
}

impl Not {
    pub fn new(expression: Expression) -> Self {
        Not {
            siddhi_element: SiddhiElement::default(),
            expression: Box::new(expression),
        }
    }
}
