use siddhi_rust::core::table::{InMemoryTable, Table};
use siddhi_rust::core::stream::input::table_input_handler::TableInputHandler;
use siddhi_rust::core::event::event::Event;
use siddhi_rust::core::event::stream::stream_event::StreamEvent;
use siddhi_rust::core::query::output::InsertIntoTableProcessor;
use siddhi_rust::core::query::processor::Processor;
use siddhi_rust::core::event::complex_event::ComplexEvent;
use siddhi_rust::core::config::siddhi_app_context::SiddhiAppContext;
use siddhi_rust::core::config::siddhi_query_context::SiddhiQueryContext;
use siddhi_rust::core::config::siddhi_context::SiddhiContext;
use siddhi_rust::core::event::value::AttributeValue;
use std::sync::Arc;

#[test]
fn test_table_input_handler_add() {
    let ctx = Arc::new(SiddhiAppContext::new(
        Arc::new(SiddhiContext::default()),
        "app".to_string(),
        Arc::new(siddhi_rust::query_api::siddhi_app::SiddhiApp::new("app".to_string())),
        String::new(),
    ));
    let table: Arc<dyn Table> = Arc::new(InMemoryTable::new());
    ctx.get_siddhi_context().add_table("T1".to_string(), table.clone());
    let handler = TableInputHandler::new(table, Arc::clone(&ctx));
    handler.add(vec![Event::new_with_data(0, vec![AttributeValue::Int(5)])]);
    assert!(ctx.get_siddhi_context().get_table("T1").unwrap().contains(&[AttributeValue::Int(5)]));
}

#[test]
fn test_insert_into_table_processor() {
    let app_ctx = Arc::new(SiddhiAppContext::new(
        Arc::new(SiddhiContext::default()),
        "app".to_string(),
        Arc::new(siddhi_rust::query_api::siddhi_app::SiddhiApp::new("app".to_string())),
        String::new(),
    ));
    let table: Arc<dyn Table> = Arc::new(InMemoryTable::new());
    app_ctx.get_siddhi_context().add_table("T2".to_string(), table.clone());
    let query_ctx = Arc::new(SiddhiQueryContext::new(Arc::clone(&app_ctx), "q".to_string(), None));
    let proc = InsertIntoTableProcessor::new(table, Arc::clone(&app_ctx), Arc::clone(&query_ctx));
    let mut se = StreamEvent::new(0, 0, 0, 1);
    se.set_output_data(Some(vec![AttributeValue::Int(7)]));
    proc.process(Some(Box::new(se)));
    assert!(app_ctx.get_siddhi_context().get_table("T2").unwrap().contains(&[AttributeValue::Int(7)]));
}
