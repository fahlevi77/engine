#[path = "common/mod.rs"]
mod common;
use siddhi_rust::core::aggregation::IncrementalExecutor;
use siddhi_rust::core::util::parser::{parse_expression, ExpressionParserContext};
use siddhi_rust::core::config::{siddhi_app_context::SiddhiAppContext, siddhi_context::SiddhiContext, siddhi_query_context::SiddhiQueryContext};
use siddhi_rust::query_api::siddhi_app::SiddhiApp;
use siddhi_rust::query_api::definition::{StreamDefinition, attribute::Type as AttrType};
use siddhi_rust::core::event::stream::meta_stream_event::MetaStreamEvent;
use siddhi_rust::core::event::stream::stream_event::StreamEvent;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::query_api::expression::{Expression, variable::Variable};
use siddhi_rust::core::table::InMemoryTable;
use siddhi_rust::query_api::aggregation::time_period::Duration as TimeDuration;
use std::sync::Arc;
use std::collections::HashMap;

fn make_ctx(name: &str) -> ExpressionParserContext<'static> {
    let app_ctx = Arc::new(SiddhiAppContext::new(
        Arc::new(SiddhiContext::default()),
        "app".to_string(),
        Arc::new(SiddhiApp::new("app".to_string())),
        String::new(),
    ));
    let q_ctx = Arc::new(SiddhiQueryContext::new(Arc::clone(&app_ctx), name.to_string(), None));
    let stream_def = Arc::new(
        StreamDefinition::new("InStream".to_string())
            .attribute("value".to_string(), AttrType::INT),
    );
    let meta = MetaStreamEvent::new_for_single_input(Arc::clone(&stream_def));
    let mut stream_map = HashMap::new();
    stream_map.insert("InStream".to_string(), Arc::new(meta));
    let qn: &'static str = Box::leak(name.to_string().into_boxed_str());
    ExpressionParserContext {
        siddhi_app_context: Arc::clone(&app_ctx),
        siddhi_query_context: q_ctx,
        stream_meta_map: stream_map,
        table_meta_map: HashMap::new(),
        window_meta_map: HashMap::new(),
        state_meta_map: HashMap::new(),
        default_source: "InStream".to_string(),
        query_name: qn,
    }
}

#[test]
fn test_incremental_executor_basic() {
    let ctx = make_ctx("inc");
    let expr = Expression::function_no_ns(
        "sum".to_string(),
        vec![Expression::Variable(Variable::new("value".to_string()))],
    );
    let exec = parse_expression(&expr, &ctx).unwrap();
    let table = Arc::new(InMemoryTable::new());
    let mut inc = IncrementalExecutor::new(TimeDuration::Seconds, vec![exec], Box::new(|_| "key".to_string()), Arc::clone(&table));
    let mut e1 = StreamEvent::new(0,1,0,0);
    e1.before_window_data[0] = AttributeValue::Int(1);
    inc.execute(&e1);
    let mut e2 = StreamEvent::new(1500,1,0,0);
    e2.before_window_data[0] = AttributeValue::Int(2);
    inc.execute(&e2);
    // flush last bucket
    inc.execute(&StreamEvent::new(2000,1,0,0));
    let rows = table.all_rows();
    assert_eq!(rows, vec![vec![AttributeValue::Long(1)], vec![AttributeValue::Long(2)]]);
}
