// siddhi_rust/src/core/event/stream/stream_event_factory.rs
// Corresponds to io.siddhi.core.event.stream.StreamEventFactory
use super::meta_stream_event::MetaStreamEvent;
use super::stream_event::StreamEvent;

#[derive(Debug, Clone)]
pub struct StreamEventFactory {
    pub before_window_data_size: usize,
    pub on_after_window_data_size: usize,
    pub output_data_size: usize,
}

impl StreamEventFactory {
    pub fn new(before_window_data_size: usize, on_after_window_data_size: usize, output_data_size: usize) -> Self {
        Self { before_window_data_size, on_after_window_data_size, output_data_size }
    }

    pub fn from_meta(meta: &MetaStreamEvent) -> Self {
        Self {
            before_window_data_size: meta.get_before_window_data().len(),
            on_after_window_data_size: meta.get_on_after_window_data().len(),
            output_data_size: meta.get_output_data().len(),
        }
    }

    pub fn new_instance(&self) -> StreamEvent {
        StreamEvent::new(0, self.before_window_data_size, self.on_after_window_data_size, self.output_data_size)
    }
}
