use siddhi_rust::core::config::siddhi_context::SiddhiContext;
use siddhi_rust::core::event::event::Event as CoreEvent;
use siddhi_rust::core::event::value::AttributeValue as CoreAttributeValue;
use siddhi_rust::core::siddhi_app_runtime::SiddhiAppRuntime;
use siddhi_rust::core::stream::output::stream_callback::StreamCallback;
use siddhi_rust::query_api::definition::{attribute::Type as AttrType, StreamDefinition};
use siddhi_rust::query_api::execution::query::input::state::{State, StateElement};
use siddhi_rust::query_api::execution::query::input::stream::input_stream::InputStream;
use siddhi_rust::query_api::execution::query::input::stream::single_input_stream::SingleInputStream;
use siddhi_rust::query_api::execution::query::input::stream::state_input_stream::StateInputStream;
use siddhi_rust::query_api::execution::query::output::output_stream::{
    InsertIntoStreamAction, OutputStream, OutputStreamAction,
};
use siddhi_rust::query_api::execution::query::selection::{OutputAttribute, Selector};
use siddhi_rust::query_api::execution::query::Query;
use siddhi_rust::query_api::execution::ExecutionElement;
use siddhi_rust::query_api::expression::Expression;
use siddhi_rust::query_api::siddhi_app::SiddhiApp as ApiSiddhiApp;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct CollectCallback {
    events: Arc<Mutex<Vec<CoreEvent>>>,
}
impl CollectCallback {
    fn new(v: Arc<Mutex<Vec<CoreEvent>>>) -> Self {
        Self { events: v }
    }
}
impl StreamCallback for CollectCallback {
    fn receive_events(&self, events: &[CoreEvent]) {
        self.events.lock().unwrap().extend_from_slice(events);
    }
}

#[test]
fn test_sequence_runtime_processing() {
    let siddhi_context = Arc::new(SiddhiContext::new());
    let mut app = ApiSiddhiApp::new("TestApp".to_string());

    let a_def =
        StreamDefinition::new("AStream".to_string()).attribute("val".to_string(), AttrType::INT);
    let b_def =
        StreamDefinition::new("BStream".to_string()).attribute("val".to_string(), AttrType::INT);
    let out_def = StreamDefinition::new("OutStream".to_string())
        .attribute("aval".to_string(), AttrType::INT)
        .attribute("bval".to_string(), AttrType::INT);
    app.stream_definition_map
        .insert("AStream".to_string(), Arc::new(a_def));
    app.stream_definition_map
        .insert("BStream".to_string(), Arc::new(b_def));
    app.stream_definition_map
        .insert("OutStream".to_string(), Arc::new(out_def));

    let a_si = SingleInputStream::new_basic("AStream".to_string(), false, false, None, Vec::new());
    let b_si = SingleInputStream::new_basic("BStream".to_string(), false, false, None, Vec::new());
    let sse1 = State::stream(a_si);
    let sse2 = State::stream(b_si);
    let next = State::next(StateElement::Stream(sse1), StateElement::Stream(sse2));
    let state_stream = StateInputStream::sequence_stream(next, None);
    let input = InputStream::State(Box::new(state_stream));

    let mut selector = Selector::new();
    selector.selection_list = vec![
        OutputAttribute::new(
            Some("aval".to_string()),
            Expression::Variable(
                siddhi_rust::query_api::expression::variable::Variable::new("val".to_string())
                    .of_stream("AStream".to_string()),
            ),
        ),
        OutputAttribute::new(
            Some("bval".to_string()),
            Expression::Variable(
                siddhi_rust::query_api::expression::variable::Variable::new("val".to_string())
                    .of_stream("BStream".to_string()),
            ),
        ),
    ];

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
    app.execution_element_list
        .push(ExecutionElement::Query(query));

    let app = Arc::new(app);
    let runtime = SiddhiAppRuntime::new(Arc::clone(&app), siddhi_context, None).expect("runtime");
    let collected = Arc::new(Mutex::new(Vec::new()));
    runtime
        .add_callback(
            "OutStream",
            Box::new(CollectCallback::new(Arc::clone(&collected))),
        )
        .unwrap();
    runtime.start();
    let a_handler = runtime.get_input_handler("AStream").unwrap();
    let b_handler = runtime.get_input_handler("BStream").unwrap();

    a_handler
        .lock()
        .unwrap()
        .send_event_with_timestamp(0, vec![CoreAttributeValue::Int(1)])
        .unwrap();
    b_handler
        .lock()
        .unwrap()
        .send_event_with_timestamp(1, vec![CoreAttributeValue::Int(2)])
        .unwrap();
    a_handler
        .lock()
        .unwrap()
        .send_event_with_timestamp(2, vec![CoreAttributeValue::Int(3)])
        .unwrap();
    b_handler
        .lock()
        .unwrap()
        .send_event_with_timestamp(3, vec![CoreAttributeValue::Int(4)])
        .unwrap();

    runtime.shutdown();

    let events = collected.lock().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].data[0], CoreAttributeValue::Int(1));
    assert_eq!(events[0].data[1], CoreAttributeValue::Int(2));
    assert_eq!(events[1].data[0], CoreAttributeValue::Int(3));
    assert_eq!(events[1].data[1], CoreAttributeValue::Int(4));
}

#[test]
fn test_every_sequence() {
    let siddhi_context = Arc::new(SiddhiContext::new());
    let mut app = ApiSiddhiApp::new("TestApp2".to_string());

    let a_def =
        StreamDefinition::new("AStream".to_string()).attribute("val".to_string(), AttrType::INT);
    let b_def =
        StreamDefinition::new("BStream".to_string()).attribute("val".to_string(), AttrType::INT);
    let out_def = StreamDefinition::new("OutStream".to_string())
        .attribute("aval".to_string(), AttrType::INT)
        .attribute("bval".to_string(), AttrType::INT);
    app.stream_definition_map
        .insert("AStream".to_string(), Arc::new(a_def));
    app.stream_definition_map
        .insert("BStream".to_string(), Arc::new(b_def));
    app.stream_definition_map
        .insert("OutStream".to_string(), Arc::new(out_def));

    let a_si = SingleInputStream::new_basic("AStream".to_string(), false, false, None, Vec::new());
    let b_si = SingleInputStream::new_basic("BStream".to_string(), false, false, None, Vec::new());
    let sse1 = State::stream(a_si);
    let sse2 = State::stream(b_si);
    let next = State::next(
        State::every(StateElement::Stream(sse1)),
        StateElement::Stream(sse2),
    );
    let state_stream = StateInputStream::sequence_stream(next, None);
    let input = InputStream::State(Box::new(state_stream));

    let mut selector = Selector::new();
    selector.selection_list = vec![
        OutputAttribute::new(
            Some("aval".to_string()),
            Expression::Variable(
                siddhi_rust::query_api::expression::variable::Variable::new("val".to_string())
                    .of_stream("AStream".to_string()),
            ),
        ),
        OutputAttribute::new(
            Some("bval".to_string()),
            Expression::Variable(
                siddhi_rust::query_api::expression::variable::Variable::new("val".to_string())
                    .of_stream("BStream".to_string()),
            ),
        ),
    ];

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
    app.execution_element_list
        .push(ExecutionElement::Query(query));

    let app = Arc::new(app);
    let runtime = SiddhiAppRuntime::new(Arc::clone(&app), siddhi_context, None).expect("runtime");
    let collected = Arc::new(Mutex::new(Vec::new()));
    runtime
        .add_callback(
            "OutStream",
            Box::new(CollectCallback::new(Arc::clone(&collected))),
        )
        .unwrap();
    runtime.start();
    let a_handler = runtime.get_input_handler("AStream").unwrap();
    let b_handler = runtime.get_input_handler("BStream").unwrap();

    a_handler
        .lock()
        .unwrap()
        .send_event_with_timestamp(0, vec![CoreAttributeValue::Int(1)])
        .unwrap();
    b_handler
        .lock()
        .unwrap()
        .send_event_with_timestamp(1, vec![CoreAttributeValue::Int(2)])
        .unwrap();
    a_handler
        .lock()
        .unwrap()
        .send_event_with_timestamp(2, vec![CoreAttributeValue::Int(3)])
        .unwrap();
    b_handler
        .lock()
        .unwrap()
        .send_event_with_timestamp(3, vec![CoreAttributeValue::Int(4)])
        .unwrap();

    runtime.shutdown();

    let events = collected.lock().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].data[0], CoreAttributeValue::Int(1));
    assert_eq!(events[0].data[1], CoreAttributeValue::Int(2));
    assert_eq!(events[1].data[0], CoreAttributeValue::Int(3));
    assert_eq!(events[1].data[1], CoreAttributeValue::Int(4));
}

#[test]
fn test_logical_and_pattern() {
    let siddhi_context = Arc::new(SiddhiContext::new());
    let mut app = ApiSiddhiApp::new("TestApp3".to_string());

    let a_def =
        StreamDefinition::new("AStream".to_string()).attribute("val".to_string(), AttrType::INT);
    let b_def =
        StreamDefinition::new("BStream".to_string()).attribute("val".to_string(), AttrType::INT);
    let out_def = StreamDefinition::new("OutStream".to_string())
        .attribute("aval".to_string(), AttrType::INT)
        .attribute("bval".to_string(), AttrType::INT);
    app.stream_definition_map
        .insert("AStream".to_string(), Arc::new(a_def));
    app.stream_definition_map
        .insert("BStream".to_string(), Arc::new(b_def));
    app.stream_definition_map
        .insert("OutStream".to_string(), Arc::new(out_def));

    let a_si = SingleInputStream::new_basic("AStream".to_string(), false, false, None, Vec::new());
    let b_si = SingleInputStream::new_basic("BStream".to_string(), false, false, None, Vec::new());
    let sse1 = State::stream(a_si);
    let sse2 = State::stream(b_si);
    let logical = State::logical_and(sse1, sse2);
    let state_stream = StateInputStream::pattern_stream(logical, None);
    let input = InputStream::State(Box::new(state_stream));

    let mut selector = Selector::new();
    selector.selection_list = vec![
        OutputAttribute::new(
            Some("aval".to_string()),
            Expression::Variable(
                siddhi_rust::query_api::expression::variable::Variable::new("val".to_string())
                    .of_stream("AStream".to_string()),
            ),
        ),
        OutputAttribute::new(
            Some("bval".to_string()),
            Expression::Variable(
                siddhi_rust::query_api::expression::variable::Variable::new("val".to_string())
                    .of_stream("BStream".to_string()),
            ),
        ),
    ];

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
    app.execution_element_list
        .push(ExecutionElement::Query(query));

    let app = Arc::new(app);
    let runtime = SiddhiAppRuntime::new(Arc::clone(&app), siddhi_context, None).expect("runtime");
    let collected = Arc::new(Mutex::new(Vec::new()));
    runtime
        .add_callback(
            "OutStream",
            Box::new(CollectCallback::new(Arc::clone(&collected))),
        )
        .unwrap();
    runtime.start();
    let a_handler = runtime.get_input_handler("AStream").unwrap();
    let b_handler = runtime.get_input_handler("BStream").unwrap();

    a_handler
        .lock()
        .unwrap()
        .send_event_with_timestamp(0, vec![CoreAttributeValue::Int(1)])
        .unwrap();
    b_handler
        .lock()
        .unwrap()
        .send_event_with_timestamp(1, vec![CoreAttributeValue::Int(2)])
        .unwrap();

    runtime.shutdown();

    let events = collected.lock().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data[0], CoreAttributeValue::Int(1));
    assert_eq!(events[0].data[1], CoreAttributeValue::Int(2));
}
