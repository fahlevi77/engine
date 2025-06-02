// siddhi_rust/src/core/event/stream/stream_event.rs
// Corresponds to io.siddhi.core.event.stream.StreamEvent
use crate::core::event::complex_event::{ComplexEvent, ComplexEventType};
use crate::core::event::value::AttributeValue;
use std::any::Any;
use std::fmt::Debug;

// Using ComplexEventType from complex_event.rs directly
// No local StreamEventType enum needed.

#[derive(Debug, Clone, Default)]
pub struct StreamEvent {
    // Fields from Java StreamEvent
    pub timestamp: i64, // Default -1 in Java
    pub output_data: Option<Vec<AttributeValue>>, // Corresponds to outputData field
    pub event_type: ComplexEventType, // Changed to use ComplexEventType directly

    // Data arrays specific to StreamEvent in Siddhi
    // Java uses Object[], here Vec<AttributeValue>
    pub before_window_data: Vec<AttributeValue>,
    pub on_after_window_data: Vec<AttributeValue>,

    // For ComplexEvent linked list
    pub next: Option<Box<dyn ComplexEvent>>, // Changed to dyn ComplexEvent for flexibility
}

impl StreamEvent {
   pub fn new(timestamp: i64, before_window_data_size: usize, on_after_window_data_size: usize, output_data_size: usize) -> Self {
       StreamEvent {
           timestamp,
           // outputData in Java StreamEvent(b,oa,o) is initialized with `new Object[outputDataSize]`
           // If output_data_size is 0, this might be None, or empty Vec. Let's use Option.
           output_data: if output_data_size > 0 { Some(vec![AttributeValue::default(); output_data_size]) } else { None },
           event_type: ComplexEventType::default(), // Defaults to Current
           before_window_data: vec![AttributeValue::default(); before_window_data_size],
           on_after_window_data: vec![AttributeValue::default(); on_after_window_data_size],
           next: None,
       }
   }
}

impl ComplexEvent for StreamEvent {
    fn get_next(&self) -> Option<&dyn ComplexEvent> {
        self.next.as_deref()
    }

    fn set_next(&mut self, next_event: Option<Box<dyn ComplexEvent>>) {
        self.next = next_event;
    }

    fn get_output_data(&self) -> Option<&[AttributeValue]> {
        self.output_data.as_deref()
    }

    fn set_output_data_at_idx(&mut self, value: AttributeValue, index: usize) -> Result<(), String> {
        if let Some(data) = self.output_data.as_mut() {
            if index < data.len() {
                data[index] = value;
                Ok(())
            } else {
                Err(format!("Index {} out of bounds for output_data with len {}", index, data.len()))
            }
        } else {
            Err("output_data is None, cannot set value".to_string())
        }
    }

    fn set_output_data_vec(&mut self, data: Option<Vec<AttributeValue>>) {
        self.output_data = data;
    }

    fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    fn set_timestamp(&mut self, timestamp: i64) {
        self.timestamp = timestamp;
    }

    fn get_event_type(&self) -> ComplexEventType {
        self.event_type // Direct access
    }

    fn set_event_type(&mut self, event_type: ComplexEventType) {
        self.event_type = event_type; // Direct assignment
    }

    // get_attribute_by_position and set_attribute_by_position are complex.
    // Java StreamEvent uses constants like BEFORE_WINDOW_DATA_INDEX.
    // These would be specific to StreamEvent's implementation of attribute access.
    // fn get_attribute_by_position(&self, position: &[i32]) -> Option<AttributeValue> { ... }
    // fn set_attribute_by_position(&mut self, value: AttributeValue, position: &[i32]) -> Result<(), String> { ... }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }
}
