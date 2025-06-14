use siddhi_rust::core::config::{
    siddhi_app_context::SiddhiAppContext, siddhi_context::SiddhiContext,
    siddhi_query_context::SiddhiQueryContext,
};
use siddhi_rust::core::event::complex_event::{ComplexEvent, ComplexEventType};
use siddhi_rust::core::event::stream::meta_stream_event::MetaStreamEvent;
use siddhi_rust::core::event::stream::stream_event::StreamEvent;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::core::util::parser::{parse_expression, ExpressionParserContext};
use siddhi_rust::query_api::definition::{attribute::Type as AttrType, StreamDefinition};
use siddhi_rust::query_api::expression::{variable::Variable, Expression};
use siddhi_rust::query_api::siddhi_app::SiddhiApp;
use std::collections::HashMap;
use std::sync::Arc;

fn make_ctx(name: &str) -> ExpressionParserContext<'static> {
    let app_ctx = Arc::new(SiddhiAppContext::new(
        Arc::new(SiddhiContext::default()),
        "app".to_string(),
        Arc::new(SiddhiApp::new("app".to_string())),
        String::new(),
    ));
    let q_ctx = Arc::new(SiddhiQueryContext::new(
        Arc::clone(&app_ctx),
        name.to_string(),
        None,
    ));
    let stream_def = Arc::new(
        StreamDefinition::new("s".to_string()).attribute("price".to_string(), AttrType::INT),
    );
    let meta = MetaStreamEvent::new_for_single_input(Arc::clone(&stream_def));
    let mut stream_map = HashMap::new();
    stream_map.insert("s".to_string(), Arc::new(meta));
    let qn: &'static str = Box::leak(name.to_string().into_boxed_str());
    ExpressionParserContext {
        siddhi_app_context: Arc::clone(&app_ctx),
        siddhi_query_context: q_ctx,
        stream_meta_map: stream_map,
        table_meta_map: HashMap::new(),
        window_meta_map: HashMap::new(),
        state_meta_map: HashMap::new(),
        default_source: "s".to_string(),
        query_name: qn,
    }
}

#[test]
fn test_sum_aggregator() {
    let ctx = make_ctx("sumq");
    let expr = Expression::function_no_ns(
        "sum".to_string(),
        vec![Expression::Variable(Variable::new("price".to_string()))],
    );
    let exec = parse_expression(&expr, &ctx).unwrap();
    // build events
    let mut e1 = StreamEvent::new(0, 1, 0, 0);
    e1.before_window_data[0] = AttributeValue::Int(5);
    let mut e2 = StreamEvent::new(0, 1, 0, 0);
    e2.before_window_data[0] = AttributeValue::Int(10);

    assert_eq!(exec.execute(Some(&e1)), Some(AttributeValue::Long(5)));
    assert_eq!(exec.execute(Some(&e2)), Some(AttributeValue::Long(15)));
    let mut reset = StreamEvent::new(0, 0, 0, 0);
    reset.set_event_type(ComplexEventType::Reset);
    exec.execute(Some(&reset));
    let mut e3 = StreamEvent::new(0, 1, 0, 0);
    e3.before_window_data[0] = AttributeValue::Int(4);
    assert_eq!(exec.execute(Some(&e3)), Some(AttributeValue::Long(4)));
}
