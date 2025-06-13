use super::input_handler::InputProcessor;
use crate::core::util::thread_barrier::ThreadBarrier;
use crate::core::event::event::Event;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct InputEntryValve {
    pub barrier: Arc<ThreadBarrier>,
    pub input_processor: Arc<Mutex<dyn InputProcessor>>,
}

impl InputEntryValve {
    pub fn new(barrier: Arc<ThreadBarrier>, input_processor: Arc<Mutex<dyn InputProcessor>>) -> Self {
        Self { barrier, input_processor }
    }
}

impl InputProcessor for InputEntryValve {
    fn send_event_with_data(&mut self, timestamp: i64, data: Vec<crate::core::event::value::AttributeValue>, stream_index: usize) -> Result<(), String> {
        self.barrier.enter();
        let res = self.input_processor.lock().map_err(|_| "processor mutex poisoned".to_string())?.send_event_with_data(timestamp, data, stream_index);
        self.barrier.exit();
        res
    }

    fn send_single_event(&mut self, event: Event, stream_index: usize) -> Result<(), String> {
        self.barrier.enter();
        let res = self.input_processor.lock().map_err(|_| "processor mutex poisoned".to_string())?.send_single_event(event, stream_index);
        self.barrier.exit();
        res
    }

    fn send_multiple_events(&mut self, events: Vec<Event>, stream_index: usize) -> Result<(), String> {
        self.barrier.enter();
        let res = self.input_processor.lock().map_err(|_| "processor mutex poisoned".to_string())?.send_multiple_events(events, stream_index);
        self.barrier.exit();
        res
    }
}
