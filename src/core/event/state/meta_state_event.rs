// siddhi_rust/src/core/event/state/meta_state_event.rs
// Corresponds to io.siddhi.core.event.state.MetaStateEvent
use crate::core::event::stream::MetaStreamEvent; // Uses MetaStreamEvent
use crate::query_api::definition::StreamDefinition; // For outputStreamDefinition
use std::sync::Arc; // If definitions are shared

// MetaStateEventAttribute is an inner class in Java, not read yet.
// For now, assuming output data attributes can be represented by query_api::Attribute
use crate::query_api::definition::Attribute as QueryApiAttribute;


#[derive(Debug, Clone, Default)]
pub struct MetaStateEvent {
    // MetaStreamEvent array, size initialized in constructor
    pub meta_stream_events: Vec<Option<MetaStreamEvent>>, // Java: MetaStreamEvent[]
    // streamEventCount is just meta_stream_events.len() or a count of Some() variants

    pub output_stream_definition: Option<Arc<StreamDefinition>>,

    // In Java, outputDataAttributes is List<MetaStateEventAttribute>.
    // MetaStateEventAttribute seems to just wrap a (streamId, Attribute) pair,
    // indicating which input stream's attribute is part of the output.
    // For simplicity, using query_api::Attribute directly as a placeholder.
    pub output_data_attributes: Option<Vec<QueryApiAttribute>>,
}

impl MetaStateEvent {
    pub fn new(size: usize) -> Self {
        let mut meta_stream_events = Vec::with_capacity(size);
        for _ in 0..size {
            meta_stream_events.push(None); // Initialize with None or default MetaStreamEvent
        }
        Self {
            meta_stream_events,
            output_stream_definition: None,
            output_data_attributes: None,
        }
    }

    // TODO: Implement methods from MetaStateEvent.java
    // get_meta_stream_event(position)
    // add_event(MetaStreamEvent)
    // add_output_data_allowing_duplicate(MetaStateEventAttribute)
    // set_output_definition(StreamDefinition)
    // clone()
}
