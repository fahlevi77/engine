// SPDX-License-Identifier: MIT OR Apache-2.0

// Corresponds to io.eventflux.query.api.execution.query.input.handler.StreamHandler
use crate::query_api::eventflux_element::EventFluxElement;
use crate::query_api::expression::Expression;

// Import specific handler types
use super::filter::Filter;
use super::stream_function::StreamFunction;
use super::window::Window as WindowHandler; // Renamed to avoid conflict

// Trait for common methods from Java's StreamHandler interface
pub trait StreamHandlerTrait {
    // Removed : EventFluxElement as EventFluxElement is directly implemented by the enum
    fn get_parameters_as_option_vec(&self) -> Option<Vec<&Expression>>; // Renamed for clarity
}

#[derive(Clone, Debug, PartialEq)]
pub enum StreamHandler {
    Filter(Filter),
    Function(StreamFunction),
    Window(Box<WindowHandler>), // Window can be larger, boxing it.
}

impl StreamHandler {
    // Accessing the composed eventflux_element from the variants
    fn eventflux_element_ref(&self) -> &EventFluxElement {
        match self {
            StreamHandler::Filter(f) => &f.eventflux_element,
            StreamHandler::Function(sf) => &sf.eventflux_element,
            StreamHandler::Window(w) => &w.eventflux_element,
        }
    }

    fn eventflux_element_mut_ref(&mut self) -> &mut EventFluxElement {
        match self {
            StreamHandler::Filter(f) => &mut f.eventflux_element,
            StreamHandler::Function(sf) => &mut sf.eventflux_element,
            StreamHandler::Window(w) => &mut w.eventflux_element,
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

// The `impl EventFluxElement for StreamHandler` block was removed because EventFluxElement is a struct, not a trait.
// Access to the underlying EventFluxElement's fields can be done via methods on StreamHandler
// that use eventflux_element_ref() or eventflux_element_mut_ref() if needed.

// Note: Individual structs (Filter, StreamFunction, WindowHandler) will need to:
// 1. Compose `eventflux_element: EventFluxElement`.
// 2. Provide a method like `get_parameters_ref_internal()` for the StreamHandlerTrait dispatch.
//    (e.g., for Filter, it would be `vec![&self.filter_expression]`).
//    (e.g., for StreamFunction/WindowHandler, it would be `self.parameters.iter().collect()` or None if empty).
