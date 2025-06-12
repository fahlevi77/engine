use siddhi_rust::query_compiler::parse;
use siddhi_rust::core::siddhi_manager::SiddhiManager;
use siddhi_rust::core::stream::output::stream_callback::StreamCallback;
use siddhi_rust::core::event::event::Event;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::core::siddhi_app_runtime::SiddhiAppRuntime;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct CollectCallback {
    events: Arc<Mutex<Vec<Vec<AttributeValue>>>>,
}

impl StreamCallback for CollectCallback {
    fn receive_events(&self, events: &[Event]) {
        let mut vec = self.events.lock().unwrap();
        for e in events {
            vec.push(e.data.clone());
        }
    }
}

pub struct AppRunner {
    runtime: Arc<SiddhiAppRuntime>,
    pub collected: Arc<Mutex<Vec<Vec<AttributeValue>>>>,
}

impl AppRunner {
    pub fn new(app_string: &str, out_stream: &str) -> Self {
        let manager = SiddhiManager::new();
        let app = parse(app_string).expect("parse");
        let runtime = manager
            .create_siddhi_app_runtime_from_api(Arc::new(app), None)
            .expect("runtime");
        let collected = Arc::new(Mutex::new(Vec::new()));
        runtime
            .add_callback(out_stream, Box::new(CollectCallback { events: Arc::clone(&collected) }))
            .expect("add cb");
        runtime.start();
        Self { runtime, collected }
    }

    pub fn new_from_api(app: siddhi_rust::query_api::siddhi_app::SiddhiApp, out_stream: &str) -> Self {
        let manager = SiddhiManager::new();
        let runtime = manager
            .create_siddhi_app_runtime_from_api(Arc::new(app), None)
            .expect("runtime");
        let collected = Arc::new(Mutex::new(Vec::new()));
        runtime
            .add_callback(out_stream, Box::new(CollectCallback { events: Arc::clone(&collected) }))
            .expect("add cb");
        runtime.start();
        Self { runtime, collected }
    }

    pub fn send(&self, stream_id: &str, data: Vec<AttributeValue>) {
        if let Some(handler) = self.runtime.get_input_handler(stream_id) {
            handler
                .lock()
                .unwrap()
                .send_event_with_timestamp(0, data)
                .unwrap();
        }
    }

    pub fn send_batch(&self, stream_id: &str, batch: Vec<Vec<AttributeValue>>) {
        if let Some(handler) = self.runtime.get_input_handler(stream_id) {
            let events: Vec<Event> = batch
                .into_iter()
                .map(|d| Event::new_with_data(0, d))
                .collect();
            handler
                .lock()
                .unwrap()
                .send_multiple_events(events)
                .unwrap();
        }
    }

    pub fn shutdown(self) -> Vec<Vec<AttributeValue>> {
        self.runtime.shutdown();
        self.collected.lock().unwrap().clone()
    }
}
