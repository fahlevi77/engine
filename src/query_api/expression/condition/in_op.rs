// Corresponds to io.siddhi.query.api.expression.condition.In
use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Expression;

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct InOp { // Renamed from In to InOp
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    // In specific fields
    pub expression: Box<Expression>,
    pub source_id: String, // Table or Window name
}

impl InOp {
    pub fn new(expression: Expression, source_id: String) -> Self {
        InOp {
            siddhi_element: SiddhiElement::default(),
            expression: Box::new(expression),
            source_id,
        }
    }
}
