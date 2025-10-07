// SPDX-License-Identifier: MIT OR Apache-2.0

// Corresponds to io.eventflux.query.api.expression.math.Add
use crate::query_api::eventflux_element::EventFluxElement;
use crate::query_api::expression::Expression; // Main Expression enum

#[derive(Clone, Debug, PartialEq)] // Removed Default
pub struct Add {
    pub eventflux_element: EventFluxElement, // Composed EventFluxElement

    // Add specific fields
    // Default for Box<Expression> would be Box::new(Expression::default_variant_if_any)
    // However, these are logically required for Add.
    pub left_value: Box<Expression>,
    pub right_value: Box<Expression>,
}

impl Add {
    pub fn new(left_value: Expression, right_value: Expression) -> Self {
        Add {
            eventflux_element: EventFluxElement::default(),
            left_value: Box::new(left_value),
            right_value: Box::new(right_value),
        }
    }
}

// Deref if needed:
// impl std::ops::Deref for Add {
//     type Target = EventFluxElement;
//     fn deref(&self) -> &Self::Target { &self.eventflux_element }
// }
// impl std::ops::DerefMut for Add {
//     fn deref_mut(&mut self) -> &mut Self::Target { &mut self.eventflux_element }
// }
