// Corresponds to io.eventflux.query.api.expression.condition.In
use crate::query_api::eventflux_element::EventFluxElement;
use crate::query_api::expression::Expression;

#[derive(Clone, Debug, PartialEq)] // Removed Default
pub struct InOp {
    // Renamed from In to InOp
    pub eventflux_element: EventFluxElement, // Composed EventFluxElement

    // In specific fields
    pub expression: Box<Expression>,
    pub source_id: String, // Table or Window name
}

impl InOp {
    pub fn new(expression: Expression, source_id: String) -> Self {
        InOp {
            eventflux_element: EventFluxElement::default(),
            expression: Box::new(expression),
            source_id,
        }
    }
}
