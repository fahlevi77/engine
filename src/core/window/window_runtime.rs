// siddhi_rust/src/core/window/window_runtime.rs

use crate::core::query::processor::Processor;
use crate::query_api::definition::WindowDefinition;
use std::sync::{Arc, Mutex};

/// Minimal runtime representation of a window.
#[derive(Debug)]
pub struct WindowRuntime {
    pub definition: Arc<WindowDefinition>,
    pub processor: Option<Arc<Mutex<dyn Processor>>>,
}

impl WindowRuntime {
    pub fn new(definition: Arc<WindowDefinition>) -> Self {
        Self {
            definition,
            processor: None,
        }
    }

    pub fn set_processor(&mut self, processor: Arc<Mutex<dyn Processor>>) {
        self.processor = Some(processor);
    }
}

