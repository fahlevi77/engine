use super::input_handler::InputProcessor;
use crate::core::config::siddhi_app_context::ThreadBarrierPlaceholder;
use crate::core::event::event::Event;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct InputEntryValve {
    pub barrier: Arc<ThreadBarrierPlaceholder>,
    pub input_processor: Arc<Mutex<dyn InputProcessor>>,
}

impl InputEntryValve {
    pub fn new(barrier: Arc<ThreadBarrierPlaceholder>, input_processor: Arc<Mutex<dyn InputProcessor>>) -> Self {
        Self { barrier, input_processor }
    }
}

impl InputProcessor for InputEntryValve {
    fn send_event_with_data(&mut self, timestamp: i64, data: Vec<crate::core::event::value::AttributeValue>, stream_index: usize) -> Result<(), String> {
        // Placeholder barrier enter/exit
        self.input_processor.lock().map_err(|_| "processor mutex poisoned".to_string())?.send_event_with_data(timestamp, data, stream_index)
    }

    fn send_single_event(&mut self, event: Event, stream_index: usize) -> Result<(), String> {
        self.input_processor.lock().map_err(|_| "processor mutex poisoned".to_string())?.send_single_event(event, stream_index)
    }

    fn send_multiple_events(&mut self, events: Vec<Event>, stream_index: usize) -> Result<(), String> {
        self.input_processor.lock().map_err(|_| "processor mutex poisoned".to_string())?.send_multiple_events(events, stream_index)
    }
}
