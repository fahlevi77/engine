// siddhi_rust/src/core/stream/input/input_handler.rs
use std::sync::{Arc, Mutex};
use crate::core::stream::stream_junction::StreamJunction;
// Using core Event struct
use crate::core::event::event::Event;
// Using core ComplexEvent trait and StreamEvent struct
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::stream::stream_event::StreamEvent;
use crate::core::config::siddhi_app_context::SiddhiAppContext;

// InputProcessor is an interface in Java.
// StreamJunction.Publisher implements InputProcessor.
// InputDistributor also implements InputProcessor.
pub trait InputProcessor: Send + Sync + std::fmt::Debug {
    // streamIndex is used to identify the target stream in a multi-stream setup (e.g. by InputDistributor)
    fn send_event_with_data(&mut self, timestamp: i64, data: Vec<crate::core::event::value::AttributeValue>, stream_index: usize) -> Result<(), String>;
    fn send_single_event(&mut self, event: Event, stream_index: usize) -> Result<(), String>;
    fn send_multiple_events(&mut self, events: Vec<Event>, stream_index: usize) -> Result<(), String>;
    // Java also has send(List<Event>, int). Vec<Event> covers this.
}


#[derive(Debug, Clone)]
pub struct InputHandler {
    stream_id: String,
    // stream_index: usize, // In Java, this is int, used by InputDistributor - Removed as not used by current new()
    // input_processor: Arc<Mutex<dyn InputProcessor>>, // Removed as not used by current new()
    siddhi_app_context: Arc<SiddhiAppContext>,
    stream_junction: Arc<Mutex<StreamJunction>>,
}

impl InputHandler {
    pub fn new(
        stream_id: String,
        siddhi_app_context: Arc<SiddhiAppContext>,
        stream_junction: Arc<Mutex<StreamJunction>>, // Target junction
    ) -> Self {
        Self {
            stream_id,
            siddhi_app_context,
            stream_junction,
            // stream_index: 0, // Default or from elsewhere if used
            // input_processor: Arc::new(Mutex::new(DummyInputProcessor{})), // Placeholder if field must exist
        }
    }

    pub fn get_stream_id(&self) -> &str { &self.stream_id }

    pub fn send_data(&self, data: Vec<crate::core::event::value::AttributeValue>) -> Result<(), String> {
        let timestamp = if self.siddhi_app_context.is_playback {
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64
        } else {
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64
        };
        let event = Event::new_with_data(timestamp, data);
        self.stream_junction.lock().expect("StreamJunction Mutex poisoned").send_event(event);
        Ok(())
    }

    pub fn send_event_with_timestamp(&self, timestamp: i64, data: Vec<crate::core::event::value::AttributeValue>) -> Result<(), String> {
        if self.siddhi_app_context.is_playback {
            // TODO: self.siddhi_app_context.getTimestampGenerator().setCurrentTimestamp(timestamp);
        }
        let event = Event::new_with_data(timestamp, data);
        self.stream_junction.lock().expect("StreamJunction Mutex poisoned").send_event(event);
        Ok(())
    }

    pub fn send_single_event(&self, event: Event) -> Result<(), String> {
        if self.siddhi_app_context.is_playback {
            // TODO: self.siddhi_app_context.getTimestampGenerator().setCurrentTimestamp(event.getTimestamp());
        }
        self.stream_junction.lock().expect("StreamJunction Mutex poisoned").send_event(event);
        Ok(())
    }

    pub fn send_multiple_events(&self, events: Vec<Event>) -> Result<(), String> {
        if self.siddhi_app_context.is_playback && !events.is_empty() {
            // TODO: self.siddhi_app_context.getTimestampGenerator().setCurrentTimestamp(events.last().unwrap().getTimestamp());
        }
        self.stream_junction.lock().expect("StreamJunction Mutex poisoned").send_events(events);
        Ok(())
    }
}

// Dummy implementation for InputProcessor placeholder if InputHandler must have one (it doesn't in this version)
// #[derive(Debug, Clone)]
// struct DummyInputProcessor;
// impl InputProcessor for DummyInputProcessor {
//     fn send_event_with_data(&mut self, _timestamp: i64, _data: Vec<crate::core::event::value::AttributeValue>, _stream_index: usize) -> Result<(), String> { Ok(()) }
//     fn send_single_event(&mut self, _event: Event, _stream_index: usize) -> Result<(), String> { Ok(()) }
//     fn send_multiple_events(&mut self, _events: Vec<Event>, _stream_index: usize) -> Result<(), String> { Ok(()) }
// }
