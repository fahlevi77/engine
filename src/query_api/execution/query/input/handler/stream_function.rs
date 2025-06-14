// Corresponds to io.siddhi.query.api.execution.query.input.handler.StreamFunction
// Implements StreamHandler and Extension in Java.
use crate::query_api::expression::Expression;
use crate::query_api::siddhi_element::SiddhiElement;

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct StreamFunction {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    // Extension fields
    pub namespace: Option<String>,
    pub name: String, // 'function' in Java, but 'name' is more generic for Extension trait

    // StreamFunction specific fields
    pub parameters: Vec<Expression>,
}

impl StreamFunction {
    // Constructor requires name. Namespace and parameters can be defaulted.
    pub fn new(name: String, namespace: Option<String>, parameters: Vec<Expression>) -> Self {
        StreamFunction {
            siddhi_element: SiddhiElement::default(),
            namespace,
            name,
            parameters,
        }
    }

    // Corresponds to getParameters()
    pub fn get_parameters(&self) -> &[Expression] {
        &self.parameters
    }

    // Helper for StreamHandlerTrait's get_parameters_as_option_vec
    pub(super) fn get_parameters_ref_internal(&self) -> Option<Vec<&Expression>> {
        if self.parameters.is_empty() {
            None
        } else {
            Some(self.parameters.iter().collect())
        }
    }
}

// Extension trait (conceptual)
// pub trait Extension {
//     fn get_namespace(&self) -> Option<&str>;
//     fn get_name(&self) -> &str;
// }
// impl Extension for StreamFunction {
//     fn get_namespace(&self) -> Option<&str> { self.namespace.as_deref() }
//     fn get_name(&self) -> &str { &self.name }
// }
