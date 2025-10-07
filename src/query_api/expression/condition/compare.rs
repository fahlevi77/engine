// Corresponds to io.eventflux.query.api.expression.condition.Compare
use crate::query_api::eventflux_element::EventFluxElement;
use crate::query_api::expression::Expression;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy, Default)] // Added Eq, Hash, Copy
pub enum Operator {
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    #[default]
    Equal,
    NotEqual,
}

// This From impl was for a placeholder Java enum, can be removed or adapted if JNI is used.
// For now, assuming it's not needed for pure Rust logic.
// impl From<io_eventflux_query_api_expression_condition_Compare_Operator> for Operator { ... }
// #[allow(non_camel_case_types)]
// enum io_eventflux_query_api_expression_condition_Compare_Operator { ... }

#[derive(Clone, Debug, PartialEq)] // Removed Default
pub struct Compare {
    pub eventflux_element: EventFluxElement, // Composed EventFluxElement

    // Compare specific fields
    pub left_expression: Box<Expression>,
    pub operator: Operator,
    pub right_expression: Box<Expression>,
}

impl Compare {
    pub fn new(
        left_expression: Expression,
        operator: Operator,
        right_expression: Expression,
    ) -> Self {
        Compare {
            eventflux_element: EventFluxElement::default(),
            left_expression: Box::new(left_expression),
            operator,
            right_expression: Box::new(right_expression),
        }
    }
}
