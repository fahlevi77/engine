// siddhi_rust/src/core/event/state/state_event.rs
// Corresponds to io.siddhi.core.event.state.StateEvent
use crate::core::event::complex_event::{ComplexEvent, ComplexEventType};
use crate::core::event::stream::StreamEvent; // StateEvent holds StreamEvents
use crate::core::event::value::AttributeValue;
use std::any::Any;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_STATE_EVENT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Default)] // Default is placeholder
pub struct StateEvent {
    // Fields from Java StateEvent
    pub stream_events: Vec<Option<StreamEvent>>, // Java: StreamEvent[], can have nulls
    pub timestamp: i64,
    pub event_type: ComplexEventType, // Corresponds to type field
    pub output_data: Option<Vec<AttributeValue>>, // Corresponds to outputData field

    // For ComplexEvent linked list
    pub next: Option<Box<dyn ComplexEvent>>,

    // Java StateEvent has an 'id' field too (long)
    pub id: u64, // Using u64 to match Event.id, though Java StateEvent.id is not static atomic
}

impl StateEvent {
    pub fn new(stream_events_size: usize, output_size: usize) -> Self {
        // Initialize stream_events with None
        let mut stream_events_vec = Vec::with_capacity(stream_events_size);
        for _ in 0..stream_events_size {
            stream_events_vec.push(None);
        }

        Self {
            stream_events: stream_events_vec,
            timestamp: -1, // Default timestamp
            event_type: ComplexEventType::default(),
            output_data: if output_size > 0 { Some(vec![AttributeValue::default(); output_size]) } else { None },
            next: None,
            id: NEXT_STATE_EVENT_ID.fetch_add(1, Ordering::Relaxed),
        }
    }
    pub fn get_stream_event(&self, position: usize) -> Option<&StreamEvent> {
        self.stream_events.get(position)?.as_ref()
    }

    pub fn set_event(&mut self, position: usize, event: StreamEvent) {
        if position < self.stream_events.len() {
            self.stream_events[position] = Some(event);
        }
    }

    pub fn add_event(&mut self, position: usize, stream_event: StreamEvent) {
        // Simplified: just append if slot empty, otherwise replace existing next chain head.
        if position >= self.stream_events.len() {
            return;
        }
        match &mut self.stream_events[position] {
            None => self.stream_events[position] = Some(stream_event),
            Some(existing) => existing.next = Some(Box::new(stream_event)),
        }
    }

    pub fn remove_last_event(&mut self, position: usize) {
        if position >= self.stream_events.len() {
            return;
        }
        if let Some(ref mut event) = self.stream_events[position] {
            // Simplified: drop the chain entirely
            event.next = None;
        }
    }
}

// Inherent methods for StateEvent
impl StateEvent {
    pub fn set_output_data_at_idx(&mut self, value: AttributeValue, index: usize) -> Result<(), String> {
        if let Some(data) = self.output_data.as_mut() {
            if index < data.len() {
                data[index] = value;
                Ok(())
            } else {
                Err(format!("Index {} out of bounds for output_data with len {}", index, data.len()))
            }
        } else {
            if index == 0 {
                let mut new_data = vec![AttributeValue::default(); index + 1];
                new_data[index] = value;
                self.output_data = Some(new_data);
                Ok(())
            } else {
                Err("output_data is None and index is not 0, cannot set value".to_string())
            }
        }
    }

    pub fn set_output_data_vec(&mut self, data: Option<Vec<AttributeValue>>) {
        self.output_data = data;
    }
}

impl ComplexEvent for StateEvent {
    fn get_next(&self) -> Option<&dyn ComplexEvent> {
        self.next.as_deref()
    }

    fn set_next(&mut self, next_event: Option<Box<dyn ComplexEvent>>) -> Option<Box<dyn ComplexEvent>> {
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

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
