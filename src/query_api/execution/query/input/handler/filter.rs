// SPDX-License-Identifier: MIT OR Apache-2.0

// Corresponds to io.eventflux.query.api.execution.query.input.handler.Filter
use crate::query_api::eventflux_element::EventFluxElement;
use crate::query_api::expression::Expression;

// Corrected structure (previous duplicate removed):
#[derive(Clone, Debug, PartialEq)]
pub struct Filter {
    pub eventflux_element: EventFluxElement,
    pub filter_expression: Expression,
}

impl Filter {
    pub fn new(filter_expression: Expression) -> Self {
        Filter {
            eventflux_element: EventFluxElement::default(),
            filter_expression,
        }
    }

    // For StreamHandlerTrait's get_parameters_as_option_vec
    pub(super) fn get_parameters_ref_internal(&self) -> Vec<&Expression> {
        vec![&self.filter_expression]
    }
}

// No Default derive for Filter as filter_expression is mandatory.
