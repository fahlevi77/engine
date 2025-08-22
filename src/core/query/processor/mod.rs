// siddhi_rust/src/core/query/processor/mod.rs
// This file now acts as the module root for the `processor` directory.
// Its content is based on the old `processor.rs` file.

use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::config::siddhi_query_context::SiddhiQueryContext;
use crate::core::event::complex_event::ComplexEvent;
// MetaStreamEvent and ApiAbstractDefinition were commented out, keep as is for now.
// use crate::core::event::stream::meta_stream_event::MetaStreamEvent;
// use crate::query_api::definition::AbstractDefinition as ApiAbstractDefinition;
// use crate::core::executor::expression_executor::ExpressionExecutor;

use std::fmt::Debug;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProcessingMode {
    #[default]
    DEFAULT,
    SLIDE,
    BATCH,
}

/// Common metadata for Processors.
#[derive(Debug, Clone)]
pub struct CommonProcessorMeta {
    pub siddhi_app_context: Arc<SiddhiAppContext>,
    pub siddhi_query_context: Arc<SiddhiQueryContext>,
    pub query_name: String,
    pub next_processor: Option<Arc<Mutex<dyn Processor>>>,
}

impl CommonProcessorMeta {
    pub fn new(app_context: Arc<SiddhiAppContext>, query_context: Arc<SiddhiQueryContext>) -> Self {
        Self {
            siddhi_app_context: app_context,
            query_name: query_context.name.clone(),
            siddhi_query_context: query_context,
            next_processor: None,
        }
    }

    pub fn get_siddhi_query_context(&self) -> Arc<SiddhiQueryContext> {
        Arc::clone(&self.siddhi_query_context)
    }
}

/// Trait for stream processors that process event chunks.
pub trait Processor: Debug + Send + Sync {
    fn process(&self, complex_event_chunk: Option<Box<dyn ComplexEvent>>);
    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>>;
    fn set_next_processor(&mut self, next_processor: Option<Arc<Mutex<dyn Processor>>>);
    fn clone_processor(&self, siddhi_query_context: &Arc<SiddhiQueryContext>)
        -> Box<dyn Processor>;
    fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext>;
    fn get_siddhi_query_context(&self) -> Arc<SiddhiQueryContext>;
    fn get_processing_mode(&self) -> ProcessingMode;
    fn is_stateful(&self) -> bool;
    
    /// Clear group states if this processor supports it (e.g., SelectProcessor)
    fn clear_group_states(&self) {
        // Default implementation does nothing
    }
}

// Declare submodules within processor directory
pub mod stream; // For StreamProcessors like FilterProcessor

// Re-export items to be accessed via `crate::core::query::processor::`
pub use self::stream::FilterProcessor; // Example re-export from stream submodule
