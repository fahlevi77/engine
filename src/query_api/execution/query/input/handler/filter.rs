// Corresponds to io.siddhi.query.api.execution.query.input.handler.Filter
use crate::query_api::expression::Expression;
use crate::query_api::siddhi_element::SiddhiElement;

// Corrected structure (previous duplicate removed):
#[derive(Clone, Debug, PartialEq)]
pub struct Filter {
    pub siddhi_element: SiddhiElement,
    pub filter_expression: Expression,
}

impl Filter {
    pub fn new(filter_expression: Expression) -> Self {
        Filter {
            siddhi_element: SiddhiElement::default(),
            filter_expression,
        }
    }

    // For StreamHandlerTrait's get_parameters_as_option_vec
    pub(super) fn get_parameters_ref_internal(&self) -> Vec<&Expression> {
        vec![&self.filter_expression]
    }
}

// No Default derive for Filter as filter_expression is mandatory.
