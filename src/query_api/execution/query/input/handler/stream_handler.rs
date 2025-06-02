// Corresponds to io.siddhi.query.api.execution.query.input.handler.StreamHandler
use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Expression;

// Import specific handler types
use super::filter::Filter;
use super::stream_function::StreamFunction;
use super::window::Window as WindowHandler; // Renamed to avoid conflict

// Trait for common methods from Java's StreamHandler interface
pub trait StreamHandlerTrait { // Removed : SiddhiElement as SiddhiElement is directly implemented by the enum
    fn get_parameters_as_option_vec(&self) -> Option<Vec<&Expression>>; // Renamed for clarity
}

#[derive(Clone, Debug, PartialEq)]
pub enum StreamHandler {
    Filter(Filter),
    Function(StreamFunction),
    Window(Box<WindowHandler>), // Window can be larger, boxing it.
}

impl StreamHandler {
    // Accessing the composed siddhi_element from the variants
    fn siddhi_element_ref(&self) -> &SiddhiElement {
        match self {
            StreamHandler::Filter(f) => &f.siddhi_element,
            StreamHandler::Function(sf) => &sf.siddhi_element,
            StreamHandler::Window(w) => &w.siddhi_element,
        }
    }

    fn siddhi_element_mut_ref(&mut self) -> &mut SiddhiElement {
        match self {
            StreamHandler::Filter(f) => &mut f.siddhi_element,
            StreamHandler::Function(sf) => &mut sf.siddhi_element,
            StreamHandler::Window(w) => &mut w.siddhi_element,
        }
    }
}

// Implement the common get_parameters method
impl StreamHandlerTrait for StreamHandler {
    fn get_parameters_as_option_vec(&self) -> Option<Vec<&Expression>> {
        match self {
            // Assuming Filter, StreamFunction, WindowHandler have a method like `get_internal_parameters_ref()`
            StreamHandler::Filter(f) => Some(f.get_parameters_ref_internal()),
            StreamHandler::Function(sf) => sf.get_parameters_ref_internal(),
            StreamHandler::Window(w) => w.get_parameters_ref_internal(),
        }
    }
}

// Implement SiddhiElement for the enum by dispatching
impl SiddhiElement for StreamHandler {
    fn query_context_start_index(&self) -> Option<(i32, i32)> {
        self.siddhi_element_ref().query_context_start_index
    }

    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) {
        self.siddhi_element_mut_ref().query_context_start_index = index;
    }

    fn query_context_end_index(&self) -> Option<(i32, i32)> {
        self.siddhi_element_ref().query_context_end_index
    }

    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) {
        self.siddhi_element_mut_ref().query_context_end_index = index;
    }
}

// Note: Individual structs (Filter, StreamFunction, WindowHandler) will need to:
// 1. Compose `siddhi_element: SiddhiElement`.
// 2. Provide a method like `get_parameters_ref_internal()` for the StreamHandlerTrait dispatch.
//    (e.g., for Filter, it would be `vec![&self.filter_expression]`).
//    (e.g., for StreamFunction/WindowHandler, it would be `self.parameters.iter().collect()` or None if empty).
