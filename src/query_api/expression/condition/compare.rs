// Corresponds to io.siddhi.query.api.expression.condition.Compare
use crate::query_api::expression::Expression;
use crate::query_api::siddhi_element::SiddhiElement;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)] // Added Eq, Hash, Copy
pub enum Operator {
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    Equal,
    NotEqual,
}

impl Default for Operator {
    fn default() -> Self {
        Operator::Equal
    } // Defaulting to Equal, could be any.
}

// This From impl was for a placeholder Java enum, can be removed or adapted if JNI is used.
// For now, assuming it's not needed for pure Rust logic.
// impl From<io_siddhi_query_api_expression_condition_Compare_Operator> for Operator { ... }
// #[allow(non_camel_case_types)]
// enum io_siddhi_query_api_expression_condition_Compare_Operator { ... }

#[derive(Clone, Debug, PartialEq)] // Removed Default
pub struct Compare {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

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
            siddhi_element: SiddhiElement::default(),
            left_expression: Box::new(left_expression),
            operator,
            right_expression: Box::new(right_expression),
        }
    }
}
