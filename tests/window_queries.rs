use siddhi_rust::core::config::{
    siddhi_app_context::SiddhiAppContext, siddhi_context::SiddhiContext,
};
use siddhi_rust::core::event::event::Event;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::core::siddhi_manager::SiddhiManager;
use siddhi_rust::core::stream::output::stream_callback::StreamCallback;
use siddhi_rust::core::stream::stream_junction::StreamJunction;
use siddhi_rust::core::util::parser::QueryParser;
use siddhi_rust::query_api::definition::attribute::Type as AttrType;
use siddhi_rust::query_api::definition::StreamDefinition;
use siddhi_rust::query_api::execution::query::input::stream::input_stream::InputStream;
use siddhi_rust::query_api::execution::query::input::stream::single_input_stream::SingleInputStream;
use siddhi_rust::query_api::execution::query::output::output_stream::{
    InsertIntoStreamAction, OutputStream, OutputStreamAction,
};
use siddhi_rust::query_api::execution::query::selection::Selector;
use siddhi_rust::query_api::execution::query::Query;
use siddhi_rust::query_api::expression::Expression;
use siddhi_rust::query_compiler::parse;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn setup_context() -> (
    Arc<SiddhiAppContext>,
    HashMap<String, Arc<Mutex<StreamJunction>>>,
) {
    let siddhi_context = Arc::new(SiddhiContext::new());
    let app = Arc::new(siddhi_rust::query_api::siddhi_app::SiddhiApp::new(
        "TestApp".to_string(),
    ));
    let app_ctx = Arc::new(SiddhiAppContext::new(
        Arc::clone(&siddhi_context),
        "TestApp".to_string(),
        Arc::clone(&app),
        String::new(),
    ));

    let in_def = Arc::new(
        StreamDefinition::new("InputStream".to_string())
            .attribute("val".to_string(), AttrType::INT),
    );
    let out_def = Arc::new(
        StreamDefinition::new("OutStream".to_string()).attribute("val".to_string(), AttrType::INT),
    );

    let in_junction = Arc::new(Mutex::new(StreamJunction::new(
        "InputStream".to_string(),
        Arc::clone(&in_def),
        Arc::clone(&app_ctx),
        1024,
        false,
        None,
    )));
    let out_junction = Arc::new(Mutex::new(StreamJunction::new(
        "OutStream".to_string(),
        Arc::clone(&out_def),
        Arc::clone(&app_ctx),
        1024,
        false,
        None,
    )));

    let mut map = HashMap::new();
    map.insert("InputStream".to_string(), in_junction);
    map.insert("OutStream".to_string(), out_junction);

    (app_ctx, map)
}

#[derive(Debug)]
struct CollectEvents {
    events: Arc<Mutex<Vec<Event>>>,
}

impl StreamCallback for CollectEvents {
    fn receive_events(&self, events: &[Event]) {
        self.events.lock().unwrap().extend_from_slice(events);
    }
}

#[test]
fn test_length_window_query_parse() {
    let (app_ctx, junctions) = setup_context();
    let si =
        SingleInputStream::new_basic("InputStream".to_string(), false, false, None, Vec::new())
            .window(None, "length".to_string(), vec![Expression::value_int(5)]);
    let input = InputStream::Single(si);
    let selector = Selector::new();
    let insert_action = InsertIntoStreamAction {
        target_id: "OutStream".to_string(),
        is_inner_stream: false,
        is_fault_stream: false,
    };
    let out_stream = OutputStream::new(OutputStreamAction::InsertInto(insert_action), None);
    let query = Query::query()
        .from(input)
        .select(selector)
        .out_stream(out_stream);

    let res = QueryParser::parse_query(
        &query,
        &app_ctx,
        &junctions,
        &HashMap::new(),
        &HashMap::new(),
    );
    assert!(res.is_ok());
}

#[test]
fn test_time_window_query_parse() {
    let (app_ctx, junctions) = setup_context();
    let si =
        SingleInputStream::new_basic("InputStream".to_string(), false, false, None, Vec::new())
            .window(None, "time".to_string(), vec![Expression::time_sec(1)]);
    let input = InputStream::Single(si);
    let selector = Selector::new();
    let insert_action = InsertIntoStreamAction {
        target_id: "OutStream".to_string(),
        is_inner_stream: false,
        is_fault_stream: false,
    };
    let out_stream = OutputStream::new(OutputStreamAction::InsertInto(insert_action), None);
    let query = Query::query()
        .from(input)
        .select(selector)
        .out_stream(out_stream);

    let res = QueryParser::parse_query(
        &query,
        &app_ctx,
        &junctions,
        &HashMap::new(),
        &HashMap::new(),
    );
    assert!(res.is_ok());
}

#[test]
fn test_length_window_runtime() {
    let app = "\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#length(2) select v insert into Out;\n";
    let manager = SiddhiManager::new();
    let api = parse(app).expect("parse");
    let runtime = manager
        .create_siddhi_app_runtime_from_api(Arc::new(api), None)
        .expect("runtime");
    let collected = Arc::new(Mutex::new(Vec::new()));
    runtime
        .add_callback(
            "Out",
            Box::new(CollectEvents {
                events: Arc::clone(&collected),
            }),
        )
        .unwrap();
    runtime.start();
    let handler = runtime.get_input_handler("In").unwrap();
    handler
        .lock()
        .unwrap()
        .send_data(vec![AttributeValue::Int(1)])
        .unwrap();
    handler
        .lock()
        .unwrap()
        .send_data(vec![AttributeValue::Int(2)])
        .unwrap();
    handler
        .lock()
        .unwrap()
        .send_data(vec![AttributeValue::Int(3)])
        .unwrap();
    std::thread::sleep(Duration::from_millis(50));
    runtime.shutdown();
    let out = collected.lock().unwrap().clone();
    assert_eq!(out.len(), 4);
    assert_eq!(out[0].data, vec![AttributeValue::Int(1)]);
    assert_eq!(out[1].data, vec![AttributeValue::Int(2)]);
    assert!(out[2].is_expired);
    assert_eq!(out[2].data, vec![AttributeValue::Int(1)]);
    assert_eq!(out[3].data, vec![AttributeValue::Int(3)]);
}

#[test]
fn test_time_window_runtime() {
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
            Box::new(CollectEvents {
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
    assert!(out.len() >= 2);
    assert_eq!(out[0].data, vec![AttributeValue::Int(5)]);
    assert!(out.iter().any(|e| e.is_expired));
}

#[test]
fn test_length_batch_window_runtime() {
    let app = "\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#lengthBatch(2) select v insert into Out;\n";
    let manager = SiddhiManager::new();
    let api = parse(app).expect("parse");
    let runtime = manager
        .create_siddhi_app_runtime_from_api(Arc::new(api), None)
        .expect("runtime");
    let collected = Arc::new(Mutex::new(Vec::new()));
    runtime
        .add_callback(
            "Out",
            Box::new(CollectEvents {
                events: Arc::clone(&collected),
            }),
        )
        .unwrap();
    runtime.start();
    let handler = runtime.get_input_handler("In").unwrap();
    handler.lock().unwrap().send_data(vec![AttributeValue::Int(1)]).unwrap();
    handler.lock().unwrap().send_data(vec![AttributeValue::Int(2)]).unwrap();
    handler.lock().unwrap().send_data(vec![AttributeValue::Int(3)]).unwrap();
    handler.lock().unwrap().send_data(vec![AttributeValue::Int(4)]).unwrap();
    std::thread::sleep(Duration::from_millis(50));
    runtime.shutdown();
    let out = collected.lock().unwrap().clone();
    assert!(out.len() >= 6);
    assert_eq!(out[0].data, vec![AttributeValue::Int(1)]);
    assert_eq!(out[1].data, vec![AttributeValue::Int(2)]);
    assert!(out[2].is_expired);
    assert!(out[3].is_expired);
    assert_eq!(out[4].data, vec![AttributeValue::Int(3)]);
    assert_eq!(out[5].data, vec![AttributeValue::Int(4)]);
}

#[test]
fn test_time_batch_window_runtime() {
    let app = "\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#timeBatch(100) select v insert into Out;\n";
    let manager = SiddhiManager::new();
    let api = parse(app).expect("parse");
    let runtime = manager
        .create_siddhi_app_runtime_from_api(Arc::new(api), None)
        .expect("runtime");
    let collected = Arc::new(Mutex::new(Vec::new()));
    runtime
        .add_callback(
            "Out",
            Box::new(CollectEvents {
                events: Arc::clone(&collected),
            }),
        )
        .unwrap();
    runtime.start();
    let handler = runtime.get_input_handler("In").unwrap();
    handler.lock().unwrap().send_data(vec![AttributeValue::Int(1)]).unwrap();
    std::thread::sleep(Duration::from_millis(120));
    handler.lock().unwrap().send_data(vec![AttributeValue::Int(2)]).unwrap();
    std::thread::sleep(Duration::from_millis(120));
    runtime.shutdown();
    let out = collected.lock().unwrap().clone();
    assert!(out.len() >= 3);
    assert_eq!(out[0].data, vec![AttributeValue::Int(1)]);
    assert!(out.iter().any(|e| e.is_expired));
}
