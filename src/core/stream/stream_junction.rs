// siddhi_rust/src/core/stream/stream_junction.rs
// Corresponds to io.siddhi.core.stream.StreamJunction
use std::sync::{Arc, Mutex};
use std::fmt::Debug;
use crossbeam_channel::{bounded, Sender, Receiver as CrossbeamReceiver, SendError, TrySendError, RecvError}; // For event buffer, aliased Receiver
use crate::core::config::siddhi_app_context::SiddhiAppContext; // Actual struct
use crate::core::event::event::Event; // Actual struct
use crate::core::event::complex_event::ComplexEvent; // Trait
use crate::core::event::stream::StreamEvent; // Actual struct for conversion
use crate::core::query::processor::Processor; // Trait
use crate::core::util::executor_service::ExecutorServicePlaceholder as ExecutorService;
use crate::core::util::metrics_placeholders::*;
use crate::query_api::definition::StreamDefinition; // From query_api

// From Java StreamJunction.OnErrorAction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OnErrorAction {
    #[default] LOG,
    STREAM,
    STORE,
    DROP, // Not in Java enum, but a common alternative to just logging
}

// Receiver trait, mirroring Java's StreamJunction.Receiver
// Processors will implement this.
pub trait Receiver: Debug + Send + Sync {
    fn get_stream_id(&self) -> &str; // Added for context, though not in Java Receiver
    fn receive_complex_event_chunk(&self, event_chunk: &mut Option<Box<dyn ComplexEvent>>); // Primary method
    // Java also has receive(Event), receive(Event[]), receive(List<Event>), receive(long, Object[])
    // These can be default methods on the trait if they all convert to ComplexEventChunk and call the primary method.
}

// StreamJunction.Publisher in Java is an inner class that implements InputProcessor.
// Here, StreamJunction itself can provide the send methods, or we can have a separate Publisher struct.
// For now, send methods are directly on StreamJunction.

/// Routes events between producers and subscribing Processors.
pub struct StreamJunction {
    pub stream_id: String,
    stream_definition: Arc<StreamDefinition>, // Added, as it's used in constructor and error handling
    siddhi_app_context: Arc<SiddhiAppContext>,
    executor_service: Arc<ExecutorService>, // For async processing
    is_async: bool,
    buffer_size: usize,

    latency_tracker: Option<LatencyTrackerPlaceholder>,
    throughput_tracker: Option<ThroughputTrackerPlaceholder>,
    // buffered_events_tracker: Option<BufferedEventsTrackerPlaceholder>, // Part of EventBufferHolder interface

    // Subscribers are Processors. Using Arc<Mutex<dyn Processor>> for shared mutable access.
    subscribers: Arc<Mutex<Vec<Arc<Mutex<dyn Processor>>>>>,

    // For async processing with internal buffer (Disruptor in Java)
    event_sender: Option<Sender<Box<dyn ComplexEvent>>>,
    // The consuming task for async would own the CrossbeamReceiver.
    // For simplicity, not storing the CrossbeamReceiver side of the channel here.
    // The executor_service would manage the task that polls the CrossbeamReceiver.

    on_error_action: OnErrorAction,
    // fault_stream_junction: Option<Arc<Mutex<StreamJunction>>>, // For fault stream redirection
    // exception_listener: Option<...>; // From SiddhiAppContext
    // is_trace_enabled: bool;
}

impl Debug for StreamJunction {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       f.debug_struct("StreamJunction")
        .field("stream_id", &self.stream_id)
        .field("is_async", &self.is_async)
        .field("buffer_size", &self.buffer_size)
        .field("subscribers_count", &self.subscribers.lock().expect("Mutex poisoned").len())
        .field("has_event_sender", &self.event_sender.is_some())
        .field("on_error_action", &self.on_error_action)
        .finish()
   }
}

impl StreamJunction {
    pub fn new(
        stream_id: String, // Added stream_id as direct parameter
        stream_definition: Arc<StreamDefinition>,
        siddhi_app_context: Arc<SiddhiAppContext>, // Moved app_context earlier
        buffer_size: usize,
        is_async: bool, // Derived from @async annotation in Java
        // executor_service is now taken from siddhi_app_context
    ) -> Self {
        // let stream_id = stream_definition.id.clone(); // No longer needed if passed directly
        let executor_service = Arc::clone(
            siddhi_app_context.executor_service.as_ref()
                .unwrap_or_else(|| &Arc::new(ExecutorService::default()))
        );
        let (sender, crossbeam_receiver_opt) = if is_async { // Renamed receiver_opt
            let (s, r) = bounded::<Box<dyn ComplexEvent>>(buffer_size);
            (Some(s), Some(r))
        } else {
            (None, None)
        };

        let junction = Self {
            stream_id, // Use passed stream_id
            stream_definition,
            siddhi_app_context: Arc::clone(&siddhi_app_context),
            executor_service, // Use executor_service from context
            is_async,
            buffer_size,
            latency_tracker: None, // TODO: Initialize from statistics_manager if enabled
            throughput_tracker: None, // TODO: Initialize
            subscribers: Arc::new(Mutex::new(Vec::new())),
            event_sender: sender,
            on_error_action: OnErrorAction::default(), // TODO: Read from annotations
            // fault_stream_junction: None, // TODO: Set up if fault stream defined
        };

        if let Some(cb_receiver) = crossbeam_receiver_opt { // Use renamed variable
            // Spawn a task for async processing if async is true
            let internal_subscribers = Arc::clone(&junction.subscribers);
            // Use the cloned executor_service for the task
            let task_executor_service = Arc::clone(&junction.executor_service);
            task_executor_service.execute(move || {
                Self::async_event_loop(cb_receiver, internal_subscribers); // Pass aliased receiver
            });
        }
        junction
    }

    pub fn get_stream_definition(&self) -> Arc<StreamDefinition> {
        Arc::clone(&self.stream_definition)
    }

    fn async_event_loop(receiver: CrossbeamReceiver<Box<dyn ComplexEvent>>, subscribers: Arc<Mutex<Vec<Arc<Mutex<dyn Processor>>>>>) { // Corrected to CrossbeamReceiver
        // TODO: Implement batching as in Java's StreamHandler if batchSize > 1
        loop {
            match receiver.recv() {
                Ok(mut event_chunk) => { // Renamed for clarity, it's a chunk head
                    let subs = subscribers.lock().expect("Mutex poisoned during async loop");
                    for subscriber_lock in subs.iter() {
                        let mut subscriber = subscriber_lock.lock().expect("Subscriber mutex poisoned");
                        // TODO: Event cloning/passing strategy for multiple subscribers.
                        // For now, this doesn't properly clone for each subscriber.
                        // If process_complex_event_chunk takes ownership or mutates, this is wrong.
                        // It should be &mut Option<Box<dyn ComplexEvent>>
                        // This needs a proper event chunk and cloning/pooling strategy.
                        subscriber.process_complex_event_chunk(&mut Some(event_chunk.clone_event_somehow_placeholder()));
                    }
                }
                Err(RecvError) => {
                    // Channel disconnected, producer (StreamJunction) likely shut down.
                    break;
                }
            }
        }
    }

    // In Java, StreamJunction has subscribe(Receiver) where Receiver is an interface.
    // Processors (like QueryFinalProcessor, StreamProcessor) implement Receiver.
    pub fn add_subscriber(&self, processor: Arc<Mutex<dyn Processor>>) {
        self.subscribers.lock().expect("Mutex poisoned").push(processor);
    }

    // send_event (from Event)
    pub fn send_event(&self, event: Event) {
        // 1. Convert Event to a concrete ComplexEvent type (e.g., StreamEvent)
        // This requires knowing the event structure (before/onAfter/output data sizes)
        // For now, assume output_data of Event maps to output_data of StreamEvent.
        // A proper StreamEventFactory or MetaStreamEvent would be used here.
        let complex_event = Box::new(StreamEvent::new(
            event.timestamp,
            0, // beforeWindowData size (unknown from plain Event)
            0, // onAfterWindowData size (unknown from plain Event)
            event.data.len(), // outputData size
        ));
        // TODO: Populate data fields of complex_event from event.data

        if let Err(e) = self.send_complex_event_chunk(Some(complex_event)) {
            // TODO: Proper error handling via faultStreamJunction or ErrorStore
            eprintln!("Error sending event to StreamJunction {}: {:?}", self.stream_id, e);
        }
    }

    // send_events (from Vec<Event>)
    pub fn send_events(&self, events: Vec<Event>) {
        if events.is_empty() { return; }
        // TODO: Convert Vec<Event> to a linked list of ComplexEvents (StreamEvents)
        // For now, sending one by one (inefficient for async if not batched by sender channel)
        for event in events {
            self.send_event(event);
        }
    }

    // Renamed from send_complex_event to indicate it can be a chunk
    pub fn send_complex_event_chunk(&self, mut complex_event_chunk: Option<Box<dyn ComplexEvent>>) -> Result<(), String> {
        if self.is_async {
            if let Some(sender) = &self.event_sender {
                if let Some(event_head) = complex_event_chunk.take() { // Sender takes ownership
                    match sender.try_send(event_head) { // Use try_send for non-blocking, or send for blocking
                        Ok(_) => Ok(()),
                        Err(TrySendError::Full(event_back)) => {
                            // TODO: Handle buffer full - backpressure, drop, log?
                            // For now, error and drop.
                            complex_event_chunk = Some(event_back); // Put it back to signal drop
                            Err(format!("Async buffer full for stream {}", self.stream_id))
                        }
                        Err(TrySendError::Disconnected(_)) => {
                            Err(format!("Async channel disconnected for stream {}", self.stream_id))
                        }
                    }
                } else { Ok(()) }
            } else {
                Err("Async junction not initialized with a sender".to_string())
            }
        } else {
            // Synchronous path
            let subs = self.subscribers.lock().expect("Mutex poisoned");
            for subscriber_lock in subs.iter() {
                let mut subscriber = subscriber_lock.lock().expect("Subscriber mutex poisoned");
                // TODO: Event cloning/passing strategy for multiple subscribers.
                // This needs a proper event chunk and cloning/pooling strategy.
                // The signature for process_complex_event_chunk takes &mut Option<Box<dyn ComplexEvent>>
                // so it can consume/replace the chunk. For multiple subscribers, each needs its "own" chunk.
                let mut chunk_for_subscriber = complex_event_chunk.clone_event_chunk_somehow_placeholder();
                subscriber.process_complex_event_chunk(&mut chunk_for_subscriber);
            }
            Ok(())
        }
    }

    // Placeholder for cloning complex event chunk
    // In a real system, this would involve an event pool and reference counting or deep cloning.
}

// TODO: This is a temporary placeholder for the complex logic of cloning a Box<dyn ComplexEvent>
// or a chunk of them. Actual implementation requires a mechanism (e.g., a `clone_box` method on `ComplexEvent` trait).
trait CloneEventChunk {
    fn clone_event_chunk_somehow_placeholder(&self) -> Self;
}
impl<T: ?Sized + ComplexEvent> CloneEventChunk for Option<Box<T>> where Box<T>: Clone { // This bound is too restrictive
    fn clone_event_chunk_somehow_placeholder(&self) -> Self {
        // This is NOT a deep clone of the trait object's data or the linked list.
        // It only works if Box<T> is Clone, which means T must be Sized and Clone.
        // Box<dyn Trait> is not Clone by default.
        // This needs a proper clone_box() method on the ComplexEvent trait.
        self.as_ref().map(|b| b.clone()) // Fails if T is dyn ComplexEvent
    }
}
