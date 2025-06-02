// Corresponds to io.siddhi.query.api.execution.query.input.handler.Filter
use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Expression;

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct Filter {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement
    pub filter_expression: Expression, // Needs default if Filter is Default. Expression enum has no default.
                                       // Making Filter::new require expression. Default derive removed.
}
// Re-evaluating Default for Filter: Since filter_expression is not Option and Expression has no Default,
// Filter cannot easily derive Default. It should be constructed with an expression.
// Removing Default derive from Filter.

// Corrected structure:
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
