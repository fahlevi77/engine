use siddhi_rust::core::util::parser::QueryParser;
use siddhi_rust::core::config::{siddhi_app_context::SiddhiAppContext, siddhi_context::SiddhiContext};
use siddhi_rust::core::stream::stream_junction::StreamJunction;
use siddhi_rust::core::query::output::callback_processor::CallbackProcessor;
use siddhi_rust::core::stream::output::stream_callback::StreamCallback;
use siddhi_rust::query_api::execution::query::{Query, input::stream::{InputStream, SingleInputStream}, selection::{Selector, OutputAttribute}, output::output_stream::{OutputStream, OutputStreamAction, InsertIntoStreamAction}};
use siddhi_rust::query_api::definition::{StreamDefinition, attribute::{Type as AttrType, Attribute}};
use siddhi_rust::query_api::expression::{Expression, variable::Variable, constant::{Constant, ConstantValueWithFloat}};
use siddhi_rust::core::event::event::Event;
use siddhi_rust::core::event::value::AttributeValue;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Debug)]
struct CollectCallback { events: Arc<Mutex<Vec<Vec<AttributeValue>>>> }
impl StreamCallback for CollectCallback {
    fn receive_events(&self, events: &[Event]) {
        let mut vec = self.events.lock().unwrap();
        for e in events { vec.push(e.data.clone()); }
    }
}

fn setup_context() -> (Arc<SiddhiAppContext>, HashMap<String, Arc<Mutex<StreamJunction>>>) {
    let siddhi_context = Arc::new(SiddhiContext::new());
    let app = Arc::new(siddhi_rust::query_api::siddhi_app::SiddhiApp::new("App".to_string()));
    let app_ctx = Arc::new(SiddhiAppContext::new(Arc::clone(&siddhi_context), "App".to_string(), Arc::clone(&app), String::new()));
    let in_def = Arc::new(StreamDefinition::new("InputStream".to_string()).attribute("a".to_string(), AttrType::INT));
    let out_def = Arc::new(StreamDefinition::new("OutStream".to_string()).attribute("a".to_string(), AttrType::INT));
    let in_j = Arc::new(Mutex::new(StreamJunction::new("InputStream".to_string(), Arc::clone(&in_def), Arc::clone(&app_ctx), 1024, false)));
    let out_j = Arc::new(Mutex::new(StreamJunction::new("OutStream".to_string(), Arc::clone(&out_def), Arc::clone(&app_ctx), 1024, false)));
    let mut map = HashMap::new();
    map.insert("InputStream".to_string(), in_j);
    map.insert("OutStream".to_string(), out_j);
    (app_ctx, map)
}

#[test]
fn test_processor_pipeline() {
    let (app_ctx, junctions) = setup_context();
    let filter_expr = Expression::Compare(Box::new(
        siddhi_rust::query_api::expression::condition::compare::Compare::new(
            Expression::Variable(Variable::new("a".to_string())),
            siddhi_rust::query_api::expression::condition::compare::Operator::GreaterThan,
            Expression::Constant(Constant::new(ConstantValueWithFloat::Int(10)))
        )));
    let si = SingleInputStream::new_basic("InputStream".to_string(), false, false, None, vec![])
        .filter(filter_expr.clone());
    let input = InputStream::Single(si);
    let mut selector = Selector::new();
    selector.selection_list.push(OutputAttribute::new(Some("a".to_string()), Expression::Variable(Variable::new("a".to_string()))));
    let insert_action = InsertIntoStreamAction { target_id: "OutStream".to_string(), is_inner_stream: false, is_fault_stream: false };
    let out_stream = OutputStream::new(OutputStreamAction::InsertInto(insert_action), None);
    let query = Query::query().from(input).select(selector).out_stream(out_stream);
    assert!(QueryParser::parse_query(&query, &app_ctx, &junctions, &HashMap::new()).is_ok());
    let collected = Arc::new(Mutex::new(Vec::new()));
    let cb = CollectCallback { events: Arc::clone(&collected) };
    let cb_proc = Arc::new(Mutex::new(CallbackProcessor::new(
        Arc::new(Mutex::new(Box::new(cb) as Box<dyn StreamCallback>)),
        Arc::clone(&app_ctx),
        Arc::new(siddhi_rust::core::config::siddhi_query_context::SiddhiQueryContext::new(Arc::clone(&app_ctx), "cb".to_string(), None)),
    )));
    junctions.get("OutStream").unwrap().lock().unwrap().subscribe(cb_proc);
    {
        let in_j = junctions.get("InputStream").unwrap();
        in_j.lock().unwrap().send_event(Event::new_with_data(0, vec![AttributeValue::Int(5)]));
        in_j.lock().unwrap().send_event(Event::new_with_data(0, vec![AttributeValue::Int(20)]));
    }
    let out = collected.lock().unwrap().clone();
    assert_eq!(out, vec![vec![AttributeValue::Int(20)]]);
}
