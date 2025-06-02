// Corresponds to io.siddhi.query.api.execution.query.input.handler.Window
// Implements StreamHandler and Extension in Java.
// To avoid naming conflict with definition::WindowDefinition, this might be named WindowHandler.
use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Expression;

#[derive(Clone, Debug, PartialEq)]
pub struct Window { // In mod.rs, this can be `pub use self::window::Window as WindowHandler;`
    // SiddhiElement fields
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,

    // Extension fields
    pub namespace: Option<String>, // Changed from String to Option for empty namespace
    pub name: String, // 'function' in Java, but 'name' makes more sense for a window's type

    // Window specific fields
    pub parameters: Vec<Expression>, // Java uses Expression[]
}

impl Window {
    pub fn new(namespace: Option<String>, name: String, parameters: Vec<Expression>) -> Self {
        Window {
            query_context_start_index: None,
            query_context_end_index: None,
            namespace,
            name,
            parameters,
        }
    }

    // Corresponds to getParameters()
    pub fn get_parameters(&self) -> &[Expression] {
        &self.parameters
    }

    // Helper for StreamHandlerTrait to return references
    pub(super) fn get_parameters_ref(&self) -> Option<Vec<&Expression>> {
         if self.parameters.is_empty() {
            None // Java getParameters can return null
        } else {
            Some(self.parameters.iter().collect())
        }
    }
}

impl SiddhiElement for Window {
    fn query_context_start_index(&self) -> Option<(i32,i32)> { self.query_context_start_index }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) { self.query_context_start_index = index; }
    fn query_context_end_index(&self) -> Option<(i32,i32)> { self.query_context_end_index }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) { self.query_context_end_index = index; }
}

// Extension trait implementation (conceptual)
// pub trait Extension {
//     fn get_namespace(&self) -> Option<&str>;
//     fn get_name(&self) -> &str;
// }
// impl Extension for Window {
//     fn get_namespace(&self) -> Option<&str> { self.namespace.as_deref() }
//     fn get_name(&self) -> &str { &self.name }
// }
