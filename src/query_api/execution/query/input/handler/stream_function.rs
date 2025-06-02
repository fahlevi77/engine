// Corresponds to io.siddhi.query.api.execution.query.input.handler.StreamFunction
// Implements StreamHandler and Extension in Java.
use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Expression;

#[derive(Clone, Debug, PartialEq)]
pub struct StreamFunction {
    // SiddhiElement fields
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,

    // Extension fields
    pub namespace: Option<String>, // Changed from String to Option for empty namespace
    pub name: String, // 'function' in Java

    // StreamFunction specific fields
    pub parameters: Vec<Expression>,
}

impl StreamFunction {
    // Constructor matching common usage, assuming default namespace if not provided.
    pub fn new(namespace: Option<String>, name: String, parameters: Vec<Expression>) -> Self {
        StreamFunction {
            query_context_start_index: None,
            query_context_end_index: None,
            namespace,
            name,
            parameters,
        }
    }

    // Corresponds to getParameters()
    pub fn get_parameters(&self) -> &[Expression] { // Returning a slice
        &self.parameters
    }

    // Helper for StreamHandlerTrait to return references
    pub(super) fn get_parameters_ref(&self) -> Option<Vec<&Expression>> {
        if self.parameters.is_empty() {
            None // Java returns null for empty parameters in some contexts
        } else {
            Some(self.parameters.iter().collect())
        }
    }
}

impl SiddhiElement for StreamFunction {
    fn query_context_start_index(&self) -> Option<(i32,i32)> { self.query_context_start_index }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) { self.query_context_start_index = index; }
    fn query_context_end_index(&self) -> Option<(i32,i32)> { self.query_context_end_index }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) { self.query_context_end_index = index; }
}

// Extension trait implementation (conceptual)
// In Rust, traits are used for shared behavior. If `Extension` was a trait:
// pub trait Extension {
//     fn get_namespace(&self) -> Option<&str>;
//     fn get_name(&self) -> &str;
// }
// impl Extension for StreamFunction {
//     fn get_namespace(&self) -> Option<&str> { self.namespace.as_deref() }
//     fn get_name(&self) -> &str { &self.name }
// }
// For now, fields are public.
