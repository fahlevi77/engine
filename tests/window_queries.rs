use siddhi_rust::core::util::parser::QueryParser;
use siddhi_rust::core::config::{siddhi_app_context::SiddhiAppContext, siddhi_context::SiddhiContext};
use siddhi_rust::core::stream::stream_junction::StreamJunction;
use siddhi_rust::query_api::execution::query::Query;
use siddhi_rust::query_api::execution::query::input::stream::single_input_stream::SingleInputStream;
use siddhi_rust::query_api::execution::query::input::stream::input_stream::InputStream;
use siddhi_rust::query_api::execution::query::selection::Selector;
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

    let in_def = Arc::new(StreamDefinition::new("InputStream".to_string()).attribute("val".to_string(), AttrType::INT));
    let out_def = Arc::new(StreamDefinition::new("OutStream".to_string()).attribute("val".to_string(), AttrType::INT));

    let in_junction = Arc::new(Mutex::new(StreamJunction::new("InputStream".to_string(), Arc::clone(&in_def), Arc::clone(&app_ctx), 1024, false)));
    let out_junction = Arc::new(Mutex::new(StreamJunction::new("OutStream".to_string(), Arc::clone(&out_def), Arc::clone(&app_ctx), 1024, false)));

    let mut map = HashMap::new();
    map.insert("InputStream".to_string(), in_junction);
    map.insert("OutStream".to_string(), out_junction);

    (app_ctx, map)
}

#[test]
fn test_length_window_query_parse() {
    let (app_ctx, junctions) = setup_context();
    let si = SingleInputStream::new_basic("InputStream".to_string(), false, false, None, Vec::new())
        .window(None, "length".to_string(), vec![Expression::value_int(5)]);
    let input = InputStream::Single(si);
    let selector = Selector::new();
    let insert_action = InsertIntoStreamAction { target_id: "OutStream".to_string(), is_inner_stream: false, is_fault_stream: false };
    let out_stream = OutputStream::new(OutputStreamAction::InsertInto(insert_action), None);
    let query = Query::query().from(input).select(selector).out_stream(out_stream);

    let res = QueryParser::parse_query(&query, &app_ctx, &junctions);
    assert!(res.is_ok());
}

#[test]
fn test_time_window_query_parse() {
    let (app_ctx, junctions) = setup_context();
    let si = SingleInputStream::new_basic("InputStream".to_string(), false, false, None, Vec::new())
        .window(None, "time".to_string(), vec![Expression::time_sec(1)]);
    let input = InputStream::Single(si);
    let selector = Selector::new();
    let insert_action = InsertIntoStreamAction { target_id: "OutStream".to_string(), is_inner_stream: false, is_fault_stream: false };
    let out_stream = OutputStream::new(OutputStreamAction::InsertInto(insert_action), None);
    let query = Query::query().from(input).select(selector).out_stream(out_stream);

    let res = QueryParser::parse_query(&query, &app_ctx, &junctions);
    assert!(res.is_ok());
}

