// siddhi_rust/src/core/stream/input/input_handler.rs
use std::sync::Arc;
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
    stream_index: usize, // In Java, this is int, used by InputDistributor
    // input_processor is the next component in the chain (e.g., InputEntryValve or StreamJunction::Publisher)
    // Using Arc<Mutex<dyn InputProcessor>> for shared, mutable access to the processor.
    input_processor: Arc<Mutex<dyn InputProcessor>>,
    siddhi_app_context: Arc<SiddhiAppContext>,
    // Java's InputHandler has a 'pausedInputPublisher' which is the same as 'inputProcessor'.
    // The 'connect'/'disconnect' methods switch 'inputProcessor' between this and null.
    // In Rust, we can use Option<Arc<Mutex<dyn InputProcessor>>> for 'input_processor'.
    // For simplicity now, assuming it's always connected after construction.
    // is_connected: bool, // To manage state
}

impl InputHandler {
    pub fn new(
        stream_id: String,
        stream_index: usize,
        input_processor: Arc<Mutex<dyn InputProcessor>>,
        siddhi_app_context: Arc<SiddhiAppContext>
    ) -> Self {
        Self {
            stream_id,
            stream_index,
            input_processor,
            siddhi_app_context,
            // is_connected: true, // Assume connected on creation
        }
    }

    pub fn get_stream_id(&self) -> &str { &self.stream_id }

    // send(Object[] data) in Java defaults timestamp to System.currentTimeMillis()
    pub fn send_data(&self, data: Vec<crate::core::event::value::AttributeValue>) -> Result<(), String> {
        // TODO: Get current time for timestamp, or require it as parameter.
        // Siddhi's playback mode affects timestamp generation.
        let timestamp = if self.siddhi_app_context.is_playback {
            // This is complex: playback mode might need specific timestamp from a generator
            // For now, using current system time as a placeholder.
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64
        } else {
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64
        };
        // This needs to lock the Mutex to call methods on the trait object.
        self.input_processor.lock().expect("InputProcessor Mutex poisoned")
            .send_event_with_data(timestamp, data, self.stream_index)
    }

    pub fn send_event_with_timestamp(&self, timestamp: i64, data: Vec<crate::core::event::value::AttributeValue>) -> Result<(), String> {
        if self.siddhi_app_context.is_playback {
            // TODO: self.siddhi_app_context.getTimestampGenerator().setCurrentTimestamp(timestamp);
            // This requires TimestampGenerator on SiddhiAppContext.
        }
        self.input_processor.lock().expect("InputProcessor Mutex poisoned")
            .send_event_with_data(timestamp, data, self.stream_index)
    }

    pub fn send_single_event(&self, event: Event) -> Result<(), String> {
        if self.siddhi_app_context.is_playback {
            // TODO: self.siddhi_app_context.getTimestampGenerator().setCurrentTimestamp(event.getTimestamp());
        }
        self.input_processor.lock().expect("InputProcessor Mutex poisoned")
            .send_single_event(event, self.stream_index)
    }

    pub fn send_multiple_events(&self, events: Vec<Event>) -> Result<(), String> {
        if self.siddhi_app_context.is_playback && !events.is_empty() {
            // TODO: self.siddhi_app_context.getTimestampGenerator().setCurrentTimestamp(events.last().unwrap().getTimestamp());
        }
        self.input_processor.lock().expect("InputProcessor Mutex poisoned")
            .send_multiple_events(events, self.stream_index)
    }

    // connect() and disconnect() methods from Java InputHandler
    // These would manage an Option for input_processor or an internal 'is_connected' flag.
    // For now, assuming InputHandler is always "connected" after creation via its constructor.
    // pub fn connect(&mut self) { self.is_connected = true; /* potentially set self.input_processor */ }
    // pub fn disconnect(&mut self) { self.is_connected = false; /* potentially clear self.input_processor */ }
}
