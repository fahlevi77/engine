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

    pub fn get_meta_stream_event(&self, position: usize) -> Option<&MetaStreamEvent> {
        self.meta_stream_events.get(position)?.as_ref()
    }

    pub fn get_meta_stream_event_mut(&mut self, position: usize) -> Option<&mut MetaStreamEvent> {
        self.meta_stream_events.get_mut(position)?.as_mut()
    }

    pub fn add_event(&mut self, meta_stream_event: MetaStreamEvent) {
        self.meta_stream_events.push(Some(meta_stream_event));
    }

    pub fn add_output_data_allowing_duplicate(&mut self, attr: QueryApiAttribute) {
        match &mut self.output_data_attributes {
            Some(vec) => vec.push(attr),
            None => self.output_data_attributes = Some(vec![attr]),
        }
    }

    pub fn set_output_definition(&mut self, def: StreamDefinition) {
        self.output_stream_definition = Some(Arc::new(def));
    }

    pub fn get_output_data_attributes(&self) -> &[QueryApiAttribute] {
        self.output_data_attributes.as_deref().unwrap_or(&[])
    }

    pub fn stream_event_count(&self) -> usize {
        self.meta_stream_events.len()
    }

    pub fn clone_meta_state_event(&self) -> Self {
        let mut clone = MetaStateEvent {
            meta_stream_events: Vec::with_capacity(self.meta_stream_events.len()),
            output_stream_definition: self.output_stream_definition.clone(),
            output_data_attributes: self.output_data_attributes.clone(),
        };
        for opt in &self.meta_stream_events {
            clone.meta_stream_events.push(opt.clone());
        }
        clone
    }
}
