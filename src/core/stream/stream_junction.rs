// siddhi_rust/src/core/stream/stream_junction.rs
// Corresponds to io.siddhi.core.stream.StreamJunction
use crate::core::config::siddhi_app_context::SiddhiAppContext; // Actual struct
use crate::core::event::complex_event::ComplexEvent; // Trait
use crate::core::event::event::Event; // Actual struct
use crate::core::event::stream::StreamEvent; // Actual struct for conversion
use crate::core::exception::SiddhiError;
use crate::core::query::processor::Processor; // Trait
use crate::core::stream::input::input_handler::InputProcessor;
// TODO: ErrorStore will be used for error handling in future implementation
use crate::core::util::executor_service::ExecutorService;
use crate::core::util::metrics::*;
use crate::query_api::definition::StreamDefinition;
use crossbeam_channel::{bounded, Receiver as CrossbeamReceiver, RecvError, Sender, TrySendError};
use std::fmt::Debug;
use std::sync::{Arc, Mutex}; // From query_api

// From Java StreamJunction.OnErrorAction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OnErrorAction {
    #[default]
    LOG,
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

#[derive(Debug, Clone)]
pub struct Publisher {
    stream_junction: Arc<StreamJunction>,
}

impl Publisher {
    pub fn new(stream_junction: Arc<StreamJunction>) -> Self {
        Self { stream_junction }
    }
}

impl InputProcessor for Publisher {
    fn send_event_with_data(
        &mut self,
        timestamp: i64,
        data: Vec<crate::core::event::value::AttributeValue>,
        _stream_index: usize,
    ) -> Result<(), String> {
        let event = Event::new_with_data(timestamp, data);
        self.stream_junction.send_event(event);
        Ok(())
    }

    fn send_single_event(&mut self, event: Event, _stream_index: usize) -> Result<(), String> {
        self.stream_junction.send_event(event);
        Ok(())
    }

    fn send_multiple_events(
        &mut self,
        events: Vec<Event>,
        _stream_index: usize,
    ) -> Result<(), String> {
        self.stream_junction.send_events(events);
        Ok(())
    }
}

// StreamJunction.Publisher in Java is an inner class that implements InputProcessor.
// Here, StreamJunction itself can provide the send methods, or we can have a separate Publisher struct.
// For now, send methods are directly on StreamJunction.

/// Routes events between producers and subscribing Processors.
#[derive(Clone)]
pub struct StreamJunction {
    pub stream_id: String,
    stream_definition: Arc<StreamDefinition>, // Added, as it's used in constructor and error handling
    siddhi_app_context: Arc<SiddhiAppContext>,
    executor_service: Arc<ExecutorService>, // For async processing
    is_async: bool,
    buffer_size: usize,

    latency_tracker: Option<Arc<LatencyTracker>>,
    throughput_tracker: Option<Arc<ThroughputTracker>>,
    // buffered_events_tracker: Option<BufferedEventsTrackerPlaceholder>, // Part of EventBufferHolder interface

    // Subscribers are Processors. Using Arc<Mutex<dyn Processor>> for shared mutable access.
    subscribers: Arc<Mutex<Vec<Arc<Mutex<dyn Processor>>>>>,

    // For async processing with internal buffer (OptimizedStreamJunction with crossbeam pipeline)
    event_sender: Option<Sender<Box<dyn ComplexEvent>>>,
    // The consuming task for async would own the CrossbeamReceiver.
    // For simplicity, not storing the CrossbeamReceiver side of the channel here.
    // The executor_service would manage the task that polls the CrossbeamReceiver.
    on_error_action: OnErrorAction,
    started: bool,
    fault_stream_junction: Option<Arc<Mutex<StreamJunction>>>,
    // exception_listener: Option<...>; // From SiddhiAppContext
    // is_trace_enabled: bool;
}

impl Debug for StreamJunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamJunction")
            .field("stream_id", &self.stream_id)
            .field("is_async", &self.is_async)
            .field("buffer_size", &self.buffer_size)
            .field(
                "subscribers_count",
                &self.subscribers.lock().expect("Mutex poisoned").len(),
            )
            .field("has_event_sender", &self.event_sender.is_some())
            .field("has_fault_stream", &self.fault_stream_junction.is_some())
            .field("on_error_action", &self.on_error_action)
            .finish()
    }
}

impl StreamJunction {
    pub fn new(
        stream_id: String,
        stream_definition: Arc<StreamDefinition>,
        siddhi_app_context: Arc<SiddhiAppContext>,
        buffer_size: usize,
        is_async: bool,
        fault_stream_junction: Option<Arc<Mutex<StreamJunction>>>,
    ) -> Self {
        // let stream_id = stream_definition.id.clone(); // No longer needed if passed directly
        let executor_service = siddhi_app_context
            .executor_service
            .clone()
            .unwrap_or_else(|| Arc::new(ExecutorService::default()));
        let (sender, crossbeam_receiver_opt) = if is_async {
            // Renamed receiver_opt
            let (s, r) = bounded::<Box<dyn ComplexEvent>>(buffer_size);
            (Some(s), Some(r))
        } else {
            (None, None)
        };

        let enable_metrics = siddhi_app_context.get_root_metrics_level()
            != crate::core::config::siddhi_app_context::MetricsLevelPlaceholder::OFF;
        let id_clone_for_metrics = stream_id.clone();
        let junction = Self {
            stream_id,
            stream_definition,
            siddhi_app_context: Arc::clone(&siddhi_app_context),
            executor_service, // Use executor_service from context
            is_async,
            buffer_size,
            latency_tracker: if enable_metrics {
                Some(LatencyTracker::new(
                    &id_clone_for_metrics,
                    &siddhi_app_context,
                ))
            } else {
                None
            },
            throughput_tracker: if enable_metrics {
                Some(ThroughputTracker::new(
                    &id_clone_for_metrics,
                    &siddhi_app_context,
                ))
            } else {
                None
            },
            subscribers: Arc::new(Mutex::new(Vec::new())),
            event_sender: sender,
            on_error_action: OnErrorAction::default(),
            started: false,
            fault_stream_junction,
        };

        if let Some(cb_receiver) = crossbeam_receiver_opt {
            // Use renamed variable
            // Spawn a task for async processing if async is true
            let internal_subscribers = Arc::clone(&junction.subscribers);
            let exec_for_task = Arc::clone(&junction.executor_service);
            junction.executor_service.execute(move || {
                Self::async_event_loop(cb_receiver, internal_subscribers, exec_for_task);
            });
        }
        junction
    }

    pub fn get_stream_definition(&self) -> Arc<StreamDefinition> {
        Arc::clone(&self.stream_definition)
    }

    pub fn construct_publisher(&self) -> Publisher {
        Publisher::new(Arc::new(self.clone()))
    }

    /// Return the total number of events seen if metrics are enabled.
    pub fn total_events(&self) -> Option<u64> {
        self.throughput_tracker.as_ref().map(|t| t.total_events())
    }

    /// Average latency in nanoseconds if metrics are enabled.
    pub fn average_latency_ns(&self) -> Option<u64> {
        self.latency_tracker
            .as_ref()
            .and_then(|l| l.average_latency_ns())
    }

    fn async_event_loop(
        receiver: CrossbeamReceiver<Box<dyn ComplexEvent>>,
        subscribers: Arc<Mutex<Vec<Arc<Mutex<dyn Processor>>>>>,
        exec: Arc<ExecutorService>,
    ) {
        loop {
            match receiver.recv() {
                Ok(event_chunk) => {
                    let subs_guard = subscribers
                        .lock()
                        .expect("Mutex poisoned during async loop");
                    let subs: Vec<_> = subs_guard.iter().map(Arc::clone).collect();
                    drop(subs_guard);
                    for sub in subs {
                        let chunk_clone = crate::core::event::complex_event::clone_event_chain(
                            event_chunk.as_ref(),
                        );
                        let sub_arc = Arc::clone(&sub);
                        let pool = {
                            let ctx = sub_arc
                                .lock()
                                .expect("subscriber mutex")
                                .get_siddhi_query_context();
                            if ctx.is_partitioned() {
                                ctx.siddhi_app_context
                                    .get_siddhi_context()
                                    .executor_services
                                    .get_or_create_from_env(&ctx.partition_id, exec.pool_size())
                            } else {
                                Arc::clone(&exec)
                            }
                        };
                        pool.execute(move || {
                            sub_arc
                                .lock()
                                .expect("subscriber mutex")
                                .process(Some(chunk_clone));
                        });
                    }
                }
                Err(RecvError) => break,
            }
        }
    }

    // In Java, StreamJunction has subscribe(Receiver) where Receiver is an interface.
    // Processors (like QueryFinalProcessor, StreamProcessor) implement Receiver.
    pub fn subscribe(&self, processor: Arc<Mutex<dyn Processor>>) {
        // avoid duplicates when the same processor subscribes twice
        let mut subs = self.subscribers.lock().expect("Mutex poisoned");
        if !subs.iter().any(|p| Arc::ptr_eq(p, &processor)) {
            subs.push(processor);
        }
    }

    pub fn unsubscribe(&self, processor: &Arc<Mutex<dyn Processor>>) {
        let mut subs = self.subscribers.lock().expect("Mutex poisoned");
        subs.retain(|p| !Arc::ptr_eq(p, processor));
    }

    pub fn set_on_error_action(&mut self, action: OnErrorAction) {
        self.on_error_action = action;
    }

    /// Initialize async processing and notify callbacks.
    pub fn start_processing(&mut self) {
        if self.started {
            return;
        }
        self.started = true;
        if self.is_async && self.event_sender.is_none() {
            let (sender, receiver) = bounded::<Box<dyn ComplexEvent>>(self.buffer_size);
            self.event_sender = Some(sender);
            let subs = Arc::clone(&self.subscribers);
            let exec_clone = Arc::clone(&self.executor_service);
            self.executor_service
                .execute(move || Self::async_event_loop(receiver, subs, exec_clone));
        }
    }

    /// Stop async processing loop.
    pub fn stop_processing(&mut self) {
        self.event_sender = None;
        self.started = false;
    }

    // send_event (from Event)
    pub fn send_event(&self, event: Event) {
        let stream_event = Self::event_to_stream_event(event);
        if let Some(tracker) = &self.throughput_tracker {
            tracker.event_in();
        }
        let start = std::time::Instant::now();
        let boxed = Box::new(stream_event);
        if let Err(_e) = self.send_complex_event_chunk(Some(
            crate::core::event::complex_event::clone_event_chain(boxed.as_ref()),
        )) {
            // error handling already performed inside send_complex_event_chunk
        } else if let Some(lat) = &self.latency_tracker {
            lat.add_latency(start.elapsed());
        }
    }

    // send_events (from Vec<Event>)
    pub fn send_events(&self, events: Vec<Event>) {
        if events.is_empty() {
            return;
        }
        let mut iter = events.into_iter();
        let first = match iter.next() {
            Some(ev) => Self::event_to_stream_event(ev),
            None => return,
        };

        let mut head: Box<dyn ComplexEvent> = Box::new(first);
        let mut tail_ref = head.mut_next_ref_option();
        let mut count = 1;
        for ev in iter {
            let boxed = Box::new(Self::event_to_stream_event(ev));
            *tail_ref = Some(boxed);
            if let Some(ref mut last) = *tail_ref {
                tail_ref = last.mut_next_ref_option();
            }
            count += 1;
        }

        if let Some(tracker) = &self.throughput_tracker {
            tracker.events_in(count as u32);
        }
        let start = std::time::Instant::now();
        if let Err(_e) = self.send_complex_event_chunk(Some(
            crate::core::event::complex_event::clone_event_chain(head.as_ref()),
        )) {
            // error handling done inside
        } else if let Some(lat) = &self.latency_tracker {
            lat.add_latency(start.elapsed());
        }
    }

    fn event_to_stream_event(event: Event) -> StreamEvent {
        let mut stream_event = StreamEvent::new(event.timestamp, event.data.len(), 0, 0);
        stream_event.before_window_data = event.data;
        stream_event
    }

    // Renamed from send_complex_event to indicate it can be a chunk
    pub fn send_complex_event_chunk(
        &self,
        mut complex_event_chunk: Option<Box<dyn ComplexEvent>>,
    ) -> Result<(), SiddhiError> {
        if self.is_async {
            if let Some(sender) = &self.event_sender {
                if let Some(event_head) = complex_event_chunk.take() {
                    // Sender takes ownership
                    match sender.try_send(event_head) {
                        // Use try_send for non-blocking, or send for blocking
                        Ok(_) => Ok(()),
                        Err(TrySendError::Full(event_back)) => {
                            let err = SiddhiError::SendError {
                                message: format!("Async buffer full for stream {}", self.stream_id),
                            };
                            self.handle_error_with_event(Some(event_back), err);
                            Err(SiddhiError::SendError {
                                message: format!("Async buffer full for stream {}", self.stream_id),
                            })
                        }
                        Err(TrySendError::Disconnected(event_back)) => {
                            let err = SiddhiError::SendError {
                                message: format!(
                                    "Async channel disconnected for stream {}",
                                    self.stream_id
                                ),
                            };
                            self.handle_error_with_event(Some(event_back), err);
                            Err(SiddhiError::SendError {
                                message: format!(
                                    "Async channel disconnected for stream {}",
                                    self.stream_id
                                ),
                            })
                        }
                    }
                } else {
                    Ok(())
                }
            } else {
                let err = SiddhiError::SendError {
                    message: "Async junction not initialized with a sender".to_string(),
                };
                self.handle_error_with_event(complex_event_chunk, err);
                Err(SiddhiError::SendError {
                    message: "Async junction not initialized with a sender".to_string(),
                })
            }
        } else {
            // Synchronous path
            let subs_guard = self.subscribers.lock().expect("Mutex poisoned");
            let subs: Vec<_> = subs_guard.iter().map(Arc::clone).collect();
            drop(subs_guard);
            for (idx, subscriber_lock) in subs.iter().enumerate() {
                let chunk_for_sub = if idx == subs.len() - 1 {
                    complex_event_chunk.take()
                } else {
                    complex_event_chunk
                        .as_ref()
                        .map(|c| crate::core::event::complex_event::clone_event_chain(c.as_ref()))
                };
                subscriber_lock
                    .lock()
                    .expect("Subscriber mutex poisoned")
                    .process(chunk_for_sub);
            }
            Ok(())
        }
    }
    fn handle_error_with_event(&self, event: Option<Box<dyn ComplexEvent>>, e: SiddhiError) {
        match self.on_error_action {
            OnErrorAction::LOG => {
                eprintln!("[{}] {}", self.stream_id, e);
            }
            OnErrorAction::DROP => {}
            OnErrorAction::STREAM => {
                if let Some(fj) = &self.fault_stream_junction {
                    let _ = fj.lock().unwrap().send_complex_event_chunk(event);
                } else {
                    eprintln!("[{}] Fault stream not configured: {}", self.stream_id, e);
                }
            }
            OnErrorAction::STORE => {
                if let Some(store) = self
                    .siddhi_app_context
                    .get_siddhi_context()
                    .get_error_store()
                {
                    store.store(&self.stream_id, e);
                } else {
                    eprintln!("[{}] Error store not configured: {}", self.stream_id, e);
                }
            }
        }
    }
}
