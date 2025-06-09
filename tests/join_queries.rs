use siddhi_rust::core::util::parser::QueryParser;
use siddhi_rust::core::config::{siddhi_app_context::SiddhiAppContext, siddhi_context::SiddhiContext};
use siddhi_rust::core::stream::stream_junction::StreamJunction;
use siddhi_rust::query_api::execution::query::Query;
use siddhi_rust::query_api::execution::query::input::stream::{InputStream, SingleInputStream, JoinType};
use siddhi_rust::query_api::execution::query::selection::{Selector, OutputAttribute};
use siddhi_rust::query_api::execution::query::output::output_stream::{OutputStream, OutputStreamAction, InsertIntoStreamAction};
use siddhi_rust::query_api::definition::StreamDefinition;
use siddhi_rust::query_api::definition::attribute::Type as AttrType;
use siddhi_rust::query_api::expression::{Expression, variable::Variable};
use siddhi_rust::query_api::expression::condition::compare::Operator as CompareOp;
use siddhi_rust::core::query::output::callback_processor::CallbackProcessor;
use siddhi_rust::core::stream::output::stream_callback::StreamCallback;
use siddhi_rust::core::event::event::Event;
use siddhi_rust::core::event::value::AttributeValue;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

fn setup_context() -> (Arc<SiddhiAppContext>, HashMap<String, Arc<Mutex<StreamJunction>>>) {
    let siddhi_context = Arc::new(SiddhiContext::new());
    let app = Arc::new(siddhi_rust::query_api::siddhi_app::SiddhiApp::new("TestApp".to_string()));
    let app_ctx = Arc::new(SiddhiAppContext::new(Arc::clone(&siddhi_context), "TestApp".to_string(), Arc::clone(&app), String::new()));

    let left_def = Arc::new(StreamDefinition::new("LeftStream".to_string()).attribute("id".to_string(), AttrType::INT));
    let right_def = Arc::new(StreamDefinition::new("RightStream".to_string()).attribute("id".to_string(), AttrType::INT));
    let out_def = Arc::new(StreamDefinition::new("OutStream".to_string()).attribute("l".to_string(), AttrType::INT).attribute("r".to_string(), AttrType::INT));

    let left_junction = Arc::new(Mutex::new(StreamJunction::new("LeftStream".to_string(), Arc::clone(&left_def), Arc::clone(&app_ctx), 1024, false)));
    let right_junction = Arc::new(Mutex::new(StreamJunction::new("RightStream".to_string(), Arc::clone(&right_def), Arc::clone(&app_ctx), 1024, false)));
    let out_junction = Arc::new(Mutex::new(StreamJunction::new("OutStream".to_string(), Arc::clone(&out_def), Arc::clone(&app_ctx), 1024, false)));

    let mut map = HashMap::new();
    map.insert("LeftStream".to_string(), left_junction);
    map.insert("RightStream".to_string(), right_junction);
    map.insert("OutStream".to_string(), out_junction);

    (app_ctx, map)
}

fn build_join_query(join_type: JoinType) -> Query {
    let left = SingleInputStream::new_basic("LeftStream".to_string(), false, false, None, Vec::new());
    let right = SingleInputStream::new_basic("RightStream".to_string(), false, false, None, Vec::new());
    let cond = Expression::compare(
        Expression::Variable(Variable::new("id".to_string()).of_stream("LeftStream".to_string())),
        CompareOp::Equal,
        Expression::Variable(Variable::new("id".to_string()).of_stream("RightStream".to_string()))
    );
    let input = InputStream::join_stream(left, join_type, right, Some(cond), None, None, None);
    let mut selector = Selector::new();
    selector.selection_list = vec![
        OutputAttribute::new(Some("l".to_string()), Expression::Variable(Variable::new("id".to_string()).of_stream("LeftStream".to_string()))),
        OutputAttribute::new(Some("r".to_string()), Expression::Variable(Variable::new("id".to_string()).of_stream("RightStream".to_string()))),
    ];
    let insert_action = InsertIntoStreamAction { target_id: "OutStream".to_string(), is_inner_stream: false, is_fault_stream: false };
    let out_stream = OutputStream::new(OutputStreamAction::InsertInto(insert_action), None);
    Query::query().from(input).select(selector).out_stream(out_stream)
}

#[test]
fn test_parse_inner_join() {
    let (app_ctx, junctions) = setup_context();
    let q = build_join_query(JoinType::InnerJoin);
    assert!(QueryParser::parse_query(&q, &app_ctx, &junctions, &HashMap::new()).is_ok());
}

#[test]
fn test_parse_left_outer_join() {
    let (app_ctx, junctions) = setup_context();
    let q = build_join_query(JoinType::LeftOuterJoin);
    assert!(QueryParser::parse_query(&q, &app_ctx, &junctions, &HashMap::new()).is_ok());
}

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

fn collect_from_out_stream(
    app_ctx: &Arc<SiddhiAppContext>,
    junctions: &HashMap<String, Arc<Mutex<StreamJunction>>>,
) -> Arc<Mutex<Vec<Vec<AttributeValue>>>> {
    let out_junction = junctions.get("OutStream").unwrap().clone();
    let collected = Arc::new(Mutex::new(Vec::new()));
    let cb = CollectCallback { events: Arc::clone(&collected) };
    let cb_proc = Arc::new(Mutex::new(CallbackProcessor::new(
        Arc::new(Mutex::new(Box::new(cb) as Box<dyn StreamCallback>)),
        Arc::clone(app_ctx),
        Arc::new(siddhi_rust::core::config::siddhi_query_context::SiddhiQueryContext::new(
            Arc::clone(app_ctx),
            "callback".to_string(),
            None,
        )),
    )));
    out_junction.lock().unwrap().subscribe(cb_proc);
    collected
}

#[test]
fn test_inner_join_runtime() {
    let (app_ctx, junctions) = setup_context();
    let q = build_join_query(JoinType::InnerJoin);
    assert!(QueryParser::parse_query(&q, &app_ctx, &junctions, &HashMap::new()).is_ok());
    let collected = collect_from_out_stream(&app_ctx, &junctions);

    {
        let left = junctions.get("LeftStream").unwrap();
        left.lock().unwrap().send_event(Event::new_with_data(0, vec![AttributeValue::Int(1)]));
    }
    {
        let right = junctions.get("RightStream").unwrap();
        right.lock().unwrap().send_event(Event::new_with_data(0, vec![AttributeValue::Int(1)]));
    }

    let out = collected.lock().unwrap().clone();
    assert_eq!(out, vec![vec![AttributeValue::Int(1), AttributeValue::Int(1)]]);
}

#[test]
fn test_left_outer_join_runtime() {
    let (app_ctx, junctions) = setup_context();
    let q = build_join_query(JoinType::LeftOuterJoin);
    assert!(QueryParser::parse_query(&q, &app_ctx, &junctions, &HashMap::new()).is_ok());
    let collected = collect_from_out_stream(&app_ctx, &junctions);

    {
        let left = junctions.get("LeftStream").unwrap();
        left.lock().unwrap().send_event(Event::new_with_data(0, vec![AttributeValue::Int(2)]));
    }

    let out = collected.lock().unwrap().clone();
    assert_eq!(out, vec![vec![AttributeValue::Int(2), AttributeValue::Null]]);
}
