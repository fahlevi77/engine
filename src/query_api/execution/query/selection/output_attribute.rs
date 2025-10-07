// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::query_api::eventflux_element::EventFluxElement;
use crate::query_api::expression::Expression;
use crate::query_api::expression::Variable;

// Corrected structure (previous duplicate removed):
#[derive(Clone, Debug, PartialEq)]
pub struct OutputAttribute {
    pub eventflux_element: EventFluxElement,
    pub rename: Option<String>,
    pub expression: Expression,
}

impl OutputAttribute {
    // Constructor for `OutputAttribute(String rename, Expression expression)`
    // Making rename Option<String> as per prompt and current code.
    pub fn new(rename: Option<String>, expression: Expression) -> Self {
        OutputAttribute {
            eventflux_element: EventFluxElement::default(),
            rename,
            expression,
        }
    }

    // Constructor for `OutputAttribute(Variable variable)`
    pub fn new_from_variable(variable: Variable) -> Self {
        OutputAttribute {
            eventflux_element: EventFluxElement::default(), // Or copy from variable.eventflux_element?
            rename: Some(variable.attribute_name.clone()),
            expression: Expression::Variable(variable),
        }
    }

    pub fn get_rename(&self) -> &Option<String> {
        &self.rename
    }

    pub fn get_expression(&self) -> &Expression {
        &self.expression
    }
}

// No Default derive.
