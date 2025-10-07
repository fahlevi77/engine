// Corresponds to io.eventflux.query.api.expression.math.Mod
use crate::query_api::eventflux_element::EventFluxElement;
use crate::query_api::expression::Expression; // Main Expression enum

#[derive(Clone, Debug, PartialEq)] // Removed Default
pub struct ModOp {
    // Renamed from Mod to ModOp
    pub eventflux_element: EventFluxElement, // Composed EventFluxElement

    pub left_value: Box<Expression>,
    pub right_value: Box<Expression>,
}

impl ModOp {
    pub fn new(left_value: Expression, right_value: Expression) -> Self {
        ModOp {
            eventflux_element: EventFluxElement::default(),
            left_value: Box::new(left_value),
            right_value: Box::new(right_value),
        }
    }
}
