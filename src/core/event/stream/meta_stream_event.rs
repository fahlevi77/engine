// siddhi_rust/src/core/event/stream/meta_stream_event.rs
// Corresponds to io.siddhi.core.event.stream.MetaStreamEvent
use crate::query_api::definition::{Attribute, StreamDefinition, AbstractDefinition}; // From query_api
use std::sync::Arc; // If definitions are shared

// MetaStreamEvent in Java has EventType enum (TABLE, WINDOW, AGGREGATE, DEFAULT)
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy, Default)]
pub enum MetaStreamEventType {
    #[default] DEFAULT,
    TABLE,
    WINDOW,
    AGGREGATE,
}

#[derive(Debug, Clone, Default)]
pub struct MetaStreamEvent {
    // Using Vec<Attribute> directly, as in Java
    pub before_window_data: Vec<Attribute>,
    pub on_after_window_data: Option<Vec<Attribute>>, // Initialized on demand in Java
    pub output_data: Option<Vec<Attribute>>,          // Initialized on demand in Java

    // Using Arc for definitions as they are likely shared across multiple meta events
    pub input_definitions: Vec<Arc<StreamDefinition>>, // Java uses AbstractDefinition, but usually they are StreamDefinition for stream events
    pub input_reference_id: Option<String>,
    pub output_stream_definition: Option<Arc<StreamDefinition>>,

    pub event_type: MetaStreamEventType,
    pub is_multi_value: bool, // Java: multiValue
}

impl MetaStreamEvent {
    pub fn new() -> Self {
        Default::default() // Relies on Default derive for Vec and Option
    }

    // TODO: Implement methods from MetaStreamEvent.java
    // initialize_on_after_window_data()
    // add_data(Attribute) -> i32 (returns constant indicating where it was added)
    // add_output_data(Attribute)
    // add_output_data_allowing_duplicate(Attribute)
    // add_input_definition(AbstractDefinition) -> should take Arc<StreamDefinition>
    // set_output_definition(StreamDefinition) -> should take Arc<StreamDefinition>
    // get_last_input_definition() -> Option<Arc<StreamDefinition>>
    // clone() -> MetaStreamEvent (derive Clone is fine for this structure if Arc is used)
}

// Note: The distinction between AbstractDefinition and StreamDefinition for input_definitions
// needs to be handled. If other definition types (TableDefinition etc.) can be "input"
// to a stream processing path represented by MetaStreamEvent, then a Definition enum from query_api
// might be needed here, wrapped in Arc. For now, assuming StreamDefinition for simplicity.
