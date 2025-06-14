use siddhi_rust::core::config::siddhi_app_context::SiddhiAppContext;
use siddhi_rust::core::config::siddhi_context::SiddhiContext;
use siddhi_rust::core::config::siddhi_query_context::SiddhiQueryContext;
use siddhi_rust::core::query::output::callback_processor::CallbackProcessor;
use siddhi_rust::core::stream::input::input_handler::InputHandler;
use siddhi_rust::core::stream::output::{LogSink, Sink, StreamCallback};
use siddhi_rust::core::stream::{Source, StreamJunction, TimerSource};
use siddhi_rust::query_api::definition::attribute::Type as AttrType;
use siddhi_rust::query_api::definition::stream_definition::StreamDefinition;
use siddhi_rust::query_api::siddhi_app::SiddhiApp;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn timer_source_to_log_sink() {
    let ctx = Arc::new(SiddhiContext::new());
    let app = Arc::new(SiddhiApp::new("TestApp".to_string()));
    let app_ctx = Arc::new(SiddhiAppContext::new(
        Arc::clone(&ctx),
        "Test".to_string(),
        Arc::clone(&app),
        String::new(),
    ));

    let stream_def = Arc::new(
        StreamDefinition::new("FooStream".to_string())
            .attribute("message".to_string(), AttrType::STRING),
    );
    let junction = Arc::new(Mutex::new(StreamJunction::new(
        "FooStream".to_string(),
        Arc::clone(&stream_def),
        Arc::clone(&app_ctx),
        16,
        false,
        None,
    )));

    let publisher = junction.lock().unwrap().construct_publisher();
    let input_handler = Arc::new(Mutex::new(InputHandler::new(
        "FooStream".to_string(),
        0,
        Arc::new(Mutex::new(publisher)),
        Arc::clone(&app_ctx),
    )));

    let sink = LogSink::new();
    let collected = sink.events.clone();
    let callback = Arc::new(Mutex::new(Box::new(sink) as Box<dyn StreamCallback>));
    let cb_processor = Arc::new(Mutex::new(CallbackProcessor::new(
        callback,
        Arc::clone(&app_ctx),
        Arc::new(SiddhiQueryContext::new(
            Arc::clone(&app_ctx),
            "cb".to_string(),
            None,
        )),
    )));
    junction.lock().unwrap().subscribe(cb_processor);

    let mut source = TimerSource::new(10);
    source.start(Arc::clone(&input_handler));
    std::thread::sleep(Duration::from_millis(50));
    source.stop();
    std::thread::sleep(Duration::from_millis(20));

    let events = collected.lock().unwrap();
    assert!(events.len() > 0);
}
