use siddhi_rust::core::util::parser::{parse_expression, ExpressionParserContext};
use siddhi_rust::query_api::expression::{Expression, Variable};
use siddhi_rust::query_api::expression::condition::compare::Operator as CompareOp;
use siddhi_rust::query_api::definition::{StreamDefinition, attribute::Type as AttrType};
use siddhi_rust::core::event::stream::meta_stream_event::MetaStreamEvent;
use siddhi_rust::core::config::siddhi_app_context::SiddhiAppContext;
use siddhi_rust::core::config::siddhi_context::SiddhiContext;
use siddhi_rust::query_api::siddhi_app::SiddhiApp;
use std::sync::Arc;
use std::collections::HashMap;

fn make_app_ctx() -> Arc<SiddhiAppContext> {
    Arc::new(SiddhiAppContext::new(
        Arc::new(SiddhiContext::default()),
        "test".to_string(),
        Arc::new(SiddhiApp::new("app".to_string())),
        String::new(),
    ))
}

#[test]
fn test_parse_expression_multi_stream_variable() {
    let stream_a = StreamDefinition::new("A".to_string())
        .attribute("val".to_string(), AttrType::INT);
    let stream_b = StreamDefinition::new("B".to_string())
        .attribute("val".to_string(), AttrType::DOUBLE);

    let meta_a = Arc::new(MetaStreamEvent::new_for_single_input(Arc::new(stream_a)));
    let meta_b = Arc::new(MetaStreamEvent::new_for_single_input(Arc::new(stream_b)));

    let mut map = HashMap::new();
    map.insert("A".to_string(), Arc::clone(&meta_a));
    map.insert("B".to_string(), Arc::clone(&meta_b));

    let ctx = ExpressionParserContext {
        siddhi_app_context: make_app_ctx(),
        stream_meta_map: map,
        table_meta_map: HashMap::new(),
        default_source: "A".to_string(),
        query_name: "Q1",
    };

    let var_a = Variable::new("val".to_string()).of_stream("A".to_string());
    let var_b = Variable::new("val".to_string()).of_stream("B".to_string());
    let expr = Expression::compare(Expression::Variable(var_a), CompareOp::LessThan, Expression::Variable(var_b));

    let exec = parse_expression(&expr, &ctx).expect("parse failed");
    assert_eq!(exec.get_return_type(), AttrType::BOOL);
}

#[test]
fn test_compare_type_coercion_int_double() {
    let ctx = ExpressionParserContext {
        siddhi_app_context: make_app_ctx(),
        stream_meta_map: HashMap::new(),
        table_meta_map: HashMap::new(),
        default_source: "dummy".to_string(),
        query_name: "Q2",
    };

    let expr = Expression::compare(
        Expression::value_int(5),
        CompareOp::LessThan,
        Expression::value_double(5.5),
    );
    let exec = parse_expression(&expr, &ctx).unwrap();
    let result = exec.execute(None);
    assert_eq!(result, Some(siddhi_rust::core::event::value::AttributeValue::Bool(true)));
}

#[test]
fn test_variable_not_found_error() {
    let stream_a = StreamDefinition::new("A".to_string())
        .attribute("val".to_string(), AttrType::INT);
    let meta_a = Arc::new(MetaStreamEvent::new_for_single_input(Arc::new(stream_a)));
    let mut map = HashMap::new();
    map.insert("A".to_string(), Arc::clone(&meta_a));

    let ctx = ExpressionParserContext {
        siddhi_app_context: make_app_ctx(),
        stream_meta_map: map,
        table_meta_map: HashMap::new(),
        default_source: "A".to_string(),
        query_name: "Q3",
    };

    let var_b = Variable::new("missing".to_string()).of_stream("A".to_string());
    let expr = Expression::Variable(var_b);
    let err = parse_expression(&expr, &ctx).unwrap_err();
    assert!(err.contains("Q3"));
}
