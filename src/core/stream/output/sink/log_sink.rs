use crate::core::event::event::Event;
use crate::core::stream::output::sink::sink_trait::Sink;
use crate::core::stream::output::stream_callback::StreamCallback;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct LogSink {
    pub events: Arc<Mutex<Vec<Event>>>,
}

impl Default for LogSink {
    fn default() -> Self {
        Self::new()
    }
}

impl LogSink {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl StreamCallback for LogSink {
    fn receive_events(&self, events: &[Event]) {
        for e in events {
            println!("{e:?}");
            self.events.lock().unwrap().push(e.clone());
        }
    }
}

impl Sink for LogSink {
    fn clone_box(&self) -> Box<dyn Sink> {
        Box::new(self.clone())
    }
}
