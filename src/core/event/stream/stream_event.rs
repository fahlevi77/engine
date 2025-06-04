// siddhi_rust/src/core/event/stream/stream_event.rs
// Corresponds to io.siddhi.core.event.stream.StreamEvent
use crate::core::event::complex_event::{ComplexEvent, ComplexEventType}; // Using ComplexEventType from complex_event.rs
use crate::core::event::value::AttributeValue;
use std::any::Any;
use std::fmt::Debug;


/// A concrete implementation of ComplexEvent for stream processing.
#[derive(Debug, Clone, Default)]
pub struct StreamEvent {
    pub timestamp: i64,
    pub output_data: Option<Vec<AttributeValue>>,
    pub event_type: ComplexEventType, // Using the canonical ComplexEventType

    pub before_window_data: Vec<AttributeValue>,
    pub on_after_window_data: Vec<AttributeValue>,

    pub next: Option<Box<dyn ComplexEvent>>,
}

impl StreamEvent {
   pub fn new(timestamp: i64, before_window_data_size: usize, on_after_window_data_size: usize, output_data_size: usize) -> Self {
       StreamEvent {
           timestamp,
           output_data: if output_data_size > 0 { Some(vec![AttributeValue::default(); output_data_size]) } else { None },
           event_type: ComplexEventType::default(), // Defaults to Current
           before_window_data: vec![AttributeValue::default(); before_window_data_size],
           on_after_window_data: vec![AttributeValue::default(); on_after_window_data_size],
           next: None,
       }
   }
   // TODO: Implement get_attribute_by_position and set_attribute_by_position from Java StreamEvent,
   // which use SiddhiConstants for position array interpretation.
}

impl ComplexEvent for StreamEvent {
    fn get_next(&self) -> Option<&dyn ComplexEvent> {
        self.next.as_deref()
    }
    fn set_next(&mut self, next_event: Option<Box<dyn ComplexEvent>>) -> Option<Box<dyn ComplexEvent>> {
        // Return the old next, as per typical linked list operations or Java's setNext.
        // However, Java's ComplexEvent.setNext is void. For easier list manipulation in Rust,
        // returning the old value can be useful if taking ownership.
        // Given the prompt changes `mut_next_ref_option`, this `set_next` should just set.
        // Let's make it return the old next to allow taking it.
        let old_next = self.next.take();
        self.next = next_event;
        old_next
    }
    fn mut_next_ref_option(&mut self) -> &mut Option<Box<dyn ComplexEvent>> {
        &mut self.next
    }

    fn get_output_data(&self) -> Option<&[AttributeValue]> {
        self.output_data.as_deref()
    }
    fn set_output_data(&mut self, data: Option<Vec<AttributeValue>>) {
        self.output_data = data;
    }
    // fn get_output_data_mut(&mut self) -> Option<&mut Vec<AttributeValue>> {
    //     self.output_data.as_mut()
    // }


    fn get_timestamp(&self) -> i64 {
        self.timestamp
    }
    fn set_timestamp(&mut self, timestamp: i64) {
        self.timestamp = timestamp;
    }

    fn get_event_type(&self) -> ComplexEventType {
        self.event_type
    }
    fn set_event_type(&mut self, event_type: ComplexEventType) {
        self.event_type = event_type;
    }

    // is_expired() and set_expired() use default trait methods from ComplexEvent.

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }

    // fn clone_complex_event(&self) -> Box<dyn ComplexEvent> {
    //     // This requires StreamEvent's `next: Option<Box<dyn ComplexEvent>>` to be handled.
    //     // If `next` is part of the clone, it implies a deep clone of the chain, which is complex.
    //     // If `next` is None in the clone, then it's simpler:
    //     // Box::new(StreamEvent { next: None, ..self.clone() })
    //     // For now, deferring full clone_complex_event implementation.
    //     Box::new(self.clone()) // This works if Box<dyn ComplexEvent> is not part of clone or handled by its own clone.
    //                           // Current StreamEvent::clone() will clone the Box<dyn ComplexEvent> if that Box is Clone.
    //                           // Box<dyn Trait> is not Clone by default.
    //                           // This needs a `clone_box` method on the trait.
    //     unimplemented!("StreamEvent::clone_complex_event requires clone_box on ComplexEvent trait")
    // }
}
