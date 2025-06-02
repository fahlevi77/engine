// Corresponds to io.siddhi.query.api.expression.math.Add
use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Expression; // Main Expression enum

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct Add {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    // Add specific fields
    // Default for Box<Expression> would be Box::new(Expression::default_variant_if_any)
    // However, these are logically required for Add.
    pub left_value: Box<Expression>,
    pub right_value: Box<Expression>,
}

impl Add {
    pub fn new(left_value: Expression, right_value: Expression) -> Self {
        Add {
            siddhi_element: SiddhiElement::default(),
            left_value: Box::new(left_value),
            right_value: Box::new(right_value),
        }
    }
}

// Deref if needed:
// impl std::ops::Deref for Add {
//     type Target = SiddhiElement;
//     fn deref(&self) -> &Self::Target { &self.siddhi_element }
// }
// impl std::ops::DerefMut for Add {
//     fn deref_mut(&mut self) -> &mut Self::Target { &mut self.siddhi_element }
// }
