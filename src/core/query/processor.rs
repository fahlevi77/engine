// siddhi_rust/src/core/query/processor.rs
// Corresponds to io.siddhi.core.query.processor.Processor and ProcessingMode

use crate::core::event::complex_event::ComplexEvent; // Assuming ComplexEvent is a trait
use std::fmt::Debug;
use std::sync::{Arc, Mutex}; // Using Mutex for interior mutability of next_processor if Processor is shared via Arc

// In Java, ProcessingMode is an enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProcessingMode {
    #[default] DEFAULT, // Default mode
    SLIDE,
    BATCH,
    // Other modes if any, e.g., GROUPBY, WINDOW for specific processor types
}

// Processor is an interface in Java. In Rust, it will be a trait.
// It's used by StreamJunction to chain subscribers.
// Subscribers are often stateful (windows, joins), so they need to process event chunks.
pub trait Processor: Debug + Send + Sync {
    // process() in Java's StreamProcessor takes ComplexEventChunk.
    // A ComplexEventChunk is often a linked list of ComplexEvents.
    // We can represent the chunk as Option<Box<dyn ComplexEvent>> being the head of the list.
    fn process_complex_event_chunk(&self, complex_event_chunk: &mut Option<Box<dyn ComplexEvent>>);
    // The &mut Option allows the processor to take the chunk or modify it.

    // getNextProcessor() and setNextProcessor(Processor)
    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>>;
    fn set_next_processor(&mut self, next_processor: Option<Arc<Mutex<dyn Processor>>>);

    // Other common methods that might be on a Processor:
    // fn clone_processor(&self, siddhi_app_context: &Arc<SiddhiAppContext>) -> Box<dyn Processor>;
    // fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext>;
    // fn is_to_clone_event_for_next_processor(&self) -> bool;
    // fn construct_next_processor_chain(
    //     &mut self,
    //     stream_event_chain: StreamEventChain, // Placeholder type
    //     processing_mode: ProcessingMode
    // );
    // fn start(&self);
    // fn stop(&self);
    // fn get_state_factory(&self) -> StateFactory; // Placeholder type
}

// Placeholder for StreamEventChain if needed by construct_next_processor_chain
// pub struct StreamEventChain;

// Placeholder for StateFactory if needed by get_state_factory
// pub struct StateFactory;
