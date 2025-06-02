use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Expression;
use crate::query_api::expression::Variable;

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct OutputAttribute {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    // OutputAttribute fields
    pub rename: Option<String>,
    // Expression cannot be Default easily, so OutputAttribute::default() will have a default Expression variant.
    // This makes Default derive problematic if Expression itself is not Default.
    // Let's assume Expression will have a usable default (e.g. Constant(Default)).
    pub expression: Expression,
}
// Reconsidering Default for OutputAttribute: if Expression is not Default, this cannot be Default.
// Expression enum does not have a Default derive.
// So, OutputAttribute cannot derive Default.

// Corrected structure:
#[derive(Clone, Debug, PartialEq)]
pub struct OutputAttribute {
    pub siddhi_element: SiddhiElement,
    pub rename: Option<String>,
    pub expression: Expression,
}

impl OutputAttribute {
    // Constructor for `OutputAttribute(String rename, Expression expression)`
    // Making rename Option<String> as per prompt and current code.
    pub fn new(rename: Option<String>, expression: Expression) -> Self {
        OutputAttribute {
            siddhi_element: SiddhiElement::default(),
            rename,
            expression,
        }
    }

    // Constructor for `OutputAttribute(Variable variable)`
    pub fn new_from_variable(variable: Variable) -> Self {
        OutputAttribute {
            siddhi_element: SiddhiElement::default(), // Or copy from variable.siddhi_element?
            rename: Some(variable.attribute_name.clone()),
            expression: Expression::Variable(variable),
        }
    }
}

// No Default derive.
