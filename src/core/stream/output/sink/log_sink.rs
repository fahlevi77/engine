use crate::core::stream::output::stream_callback::StreamCallback;
use crate::core::event::event::Event;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct LogSink;

impl StreamCallback for LogSink {
    fn receive_events(&self, events: &[Event]) {
        for e in events {
            println!("{:?}", e);
        }
    }
}
