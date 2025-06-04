use super::input_handler::InputProcessor;
use crate::core::event::event::Event;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default, Clone)]
pub struct InputDistributor {
    input_processors: Vec<Arc<Mutex<dyn InputProcessor>>>,
}

impl InputDistributor {
    pub fn add_input_processor(&mut self, processor: Arc<Mutex<dyn InputProcessor>>) {
        self.input_processors.push(processor);
    }

    pub fn clear(&mut self) {
        self.input_processors.clear();
    }
}

impl InputProcessor for InputDistributor {
    fn send_event_with_data(&mut self, timestamp: i64, data: Vec<crate::core::event::value::AttributeValue>, stream_index: usize) -> Result<(), String> {
        if let Some(proc_lock) = self.input_processors.get(stream_index) {
            proc_lock.lock().map_err(|_| "processor mutex poisoned".to_string())?.send_event_with_data(timestamp, data, stream_index)
        } else { Err("stream_index out of bounds".into()) }
    }

    fn send_single_event(&mut self, event: Event, stream_index: usize) -> Result<(), String> {
        if let Some(proc_lock) = self.input_processors.get(stream_index) {
            proc_lock.lock().map_err(|_| "processor mutex poisoned".to_string())?.send_single_event(event, stream_index)
        } else { Err("stream_index out of bounds".into()) }
    }

    fn send_multiple_events(&mut self, events: Vec<Event>, stream_index: usize) -> Result<(), String> {
        if let Some(proc_lock) = self.input_processors.get(stream_index) {
            proc_lock.lock().map_err(|_| "processor mutex poisoned".to_string())?.send_multiple_events(events, stream_index)
        } else { Err("stream_index out of bounds".into()) }
    }
}
