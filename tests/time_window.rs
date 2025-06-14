#[path = "common/mod.rs"]
mod common;
use siddhi_rust::core::event::event::Event;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::core::siddhi_manager::SiddhiManager;
use siddhi_rust::core::stream::output::stream_callback::StreamCallback;
use siddhi_rust::query_compiler::parse;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug)]
struct CollectCallback {
    events: Arc<Mutex<Vec<Event>>>,
}
impl StreamCallback for CollectCallback {
    fn receive_events(&self, events: &[Event]) {
        self.events.lock().unwrap().extend_from_slice(events);
    }
}

#[test]
fn test_time_window_expiry() {
    let app = "\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#time(100) select v insert into Out;\n";
    let manager = SiddhiManager::new();
    let api = parse(app).expect("parse");
    let runtime = manager
        .create_siddhi_app_runtime_from_api(Arc::new(api), None)
        .expect("runtime");
    let collected = Arc::new(Mutex::new(Vec::new()));
    runtime
        .add_callback(
            "Out",
            Box::new(CollectCallback {
                events: Arc::clone(&collected),
            }),
        )
        .unwrap();
    runtime.start();
    let handler = runtime.get_input_handler("In").unwrap();
    handler
        .lock()
        .unwrap()
        .send_data(vec![AttributeValue::Int(5)])
        .unwrap();
    std::thread::sleep(Duration::from_millis(150));
    runtime.shutdown();
    let out = collected.lock().unwrap().clone();
    println!("events: {:?}", out);
    assert!(out.len() >= 2);
    assert_eq!(out[0].data, vec![AttributeValue::Int(5)]);
    assert!(out.iter().any(|e| e.is_expired));
}
