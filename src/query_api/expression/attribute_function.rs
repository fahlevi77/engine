// Corresponds to io.siddhi.query.api.expression.AttributeFunction
// This also implements Extension in Java, which means it has namespace and name.
use crate::query_api::siddhi_element::SiddhiElement;
use super::Expression; // Assuming expression.rs will define the Expression enum.

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct AttributeFunction {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    // Extension fields
    pub extension_namespace: Option<String>,
    pub function_name: String,

    // AttributeFunction specific fields
    pub parameters: Vec<Expression>,
}

impl AttributeFunction {
    pub fn new(
        extension_namespace: Option<String>,
        function_name: String,
        parameters: Vec<Expression>,
    ) -> Self {
        AttributeFunction {
            siddhi_element: SiddhiElement::default(),
            extension_namespace,
            function_name,
            parameters,
        }
    }
}

// The Java `Extension` interface has `getNamespace()` and `getName()`.
// These are covered by `extension_namespace` and `function_name` fields.
// If an `Extension` trait is defined in Rust, this struct would implement it.
// pub trait Extension {
//     fn get_namespace(&self) -> Option<&str>;
//     fn get_name(&self) -> &str;
// }
// impl Extension for AttributeFunction {
//     fn get_namespace(&self) -> Option<&str> { self.extension_namespace.as_deref() }
//     fn get_name(&self) -> &str { &self.function_name }
// }

// Deref if needed:
// impl std::ops::Deref for AttributeFunction {
//     type Target = SiddhiElement;
//     fn deref(&self) -> &Self::Target { &self.siddhi_element }
// }
// impl std::ops::DerefMut for AttributeFunction {
//     fn deref_mut(&mut self) -> &mut Self::Target { &mut self.siddhi_element }
// }
