use siddhi_rust::core::util::parser::QueryParser;
use siddhi_rust::core::config::{siddhi_app_context::SiddhiAppContext, siddhi_context::SiddhiContext};
use siddhi_rust::core::stream::stream_junction::StreamJunction;
use siddhi_rust::query_api::execution::query::Query;
use siddhi_rust::query_api::execution::query::input::stream::single_input_stream::SingleInputStream;
use siddhi_rust::query_api::execution::query::input::stream::input_stream::InputStream;
use siddhi_rust::query_api::execution::query::input::stream::state_input_stream::StateInputStream;
use siddhi_rust::query_api::execution::query::input::state::{State, StateElement};
use siddhi_rust::query_api::execution::query::selection::{Selector, OutputAttribute};
use siddhi_rust::query_api::execution::query::output::output_stream::{OutputStream, OutputStreamAction, InsertIntoStreamAction};
use siddhi_rust::query_api::definition::StreamDefinition;
use siddhi_rust::query_api::definition::attribute::Type as AttrType;
use siddhi_rust::query_api::expression::Expression;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

fn setup_context() -> (Arc<SiddhiAppContext>, HashMap<String, Arc<Mutex<StreamJunction>>>) {
    let siddhi_context = Arc::new(SiddhiContext::new());
    let app = Arc::new(siddhi_rust::query_api::siddhi_app::SiddhiApp::new("TestApp".to_string()));
    let app_ctx = Arc::new(SiddhiAppContext::new(Arc::clone(&siddhi_context), "TestApp".to_string(), Arc::clone(&app), String::new()));

    let a_def = Arc::new(StreamDefinition::new("AStream".to_string()).attribute("val".to_string(), AttrType::INT));
    let b_def = Arc::new(StreamDefinition::new("BStream".to_string()).attribute("val".to_string(), AttrType::INT));
    let out_def = Arc::new(StreamDefinition::new("OutStream".to_string()).attribute("aval".to_string(), AttrType::INT).attribute("bval".to_string(), AttrType::INT));

    let a_junction = Arc::new(Mutex::new(StreamJunction::new("AStream".to_string(), Arc::clone(&a_def), Arc::clone(&app_ctx), 1024, false, None)));
    let b_junction = Arc::new(Mutex::new(StreamJunction::new("BStream".to_string(), Arc::clone(&b_def), Arc::clone(&app_ctx), 1024, false, None)));
    let out_junction = Arc::new(Mutex::new(StreamJunction::new("OutStream".to_string(), Arc::clone(&out_def), Arc::clone(&app_ctx), 1024, false, None)));

    let mut map = HashMap::new();
    map.insert("AStream".to_string(), a_junction);
    map.insert("BStream".to_string(), b_junction);
    map.insert("OutStream".to_string(), out_junction);

    (app_ctx, map)
}

fn build_sequence_query() -> Query {
    let a_si = SingleInputStream::new_basic("AStream".to_string(), false, false, None, Vec::new());
    let b_si = SingleInputStream::new_basic("BStream".to_string(), false, false, None, Vec::new());
    let sse1 = State::stream(a_si);
    let sse2 = State::stream(b_si);
    let next = State::next(StateElement::Stream(sse1), StateElement::Stream(sse2));
    let state_stream = StateInputStream::sequence_stream(next, None);
    let input = InputStream::State(Box::new(state_stream));

    let mut selector = Selector::new();
    selector.selection_list = vec![
        OutputAttribute::new(Some("aval".to_string()), Expression::variable("val".to_string())),
        OutputAttribute::new(Some("bval".to_string()), Expression::Variable(
            siddhi_rust::query_api::expression::variable::Variable::new("val".to_string()).of_stream("BStream".to_string())
        )),
    ];

    let insert_action = InsertIntoStreamAction { target_id: "OutStream".to_string(), is_inner_stream: false, is_fault_stream: false };
    let out_stream = OutputStream::new(OutputStreamAction::InsertInto(insert_action), None);
    Query::query().from(input).select(selector).out_stream(out_stream)
}

#[test]
fn test_sequence_query_parse() {
    let (app_ctx, mut junctions) = setup_context();
    let q = build_sequence_query();
    let res = QueryParser::parse_query(&q, &app_ctx, &mut junctions, &HashMap::new());
    assert!(res.is_ok());
}
