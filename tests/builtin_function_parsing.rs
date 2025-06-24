use siddhi_rust::core::config::siddhi_app_context::SiddhiAppContext;
use siddhi_rust::core::config::siddhi_context::SiddhiContext;
use siddhi_rust::core::config::siddhi_query_context::SiddhiQueryContext;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::core::util::parser::{parse_expression, ExpressionParserContext};
use siddhi_rust::query_api::definition::attribute::Type as AttrType;
use siddhi_rust::query_api::expression::Expression;
use siddhi_rust::query_api::siddhi_app::SiddhiApp;
use std::collections::HashMap;
use std::sync::Arc;

fn make_app_ctx() -> Arc<SiddhiAppContext> {
    Arc::new(SiddhiAppContext::new(
        Arc::new(SiddhiContext::default()),
        "test".to_string(),
        Arc::new(SiddhiApp::new("app".to_string())),
        String::new(),
    ))
}

fn make_query_ctx(name: &str) -> Arc<SiddhiQueryContext> {
    Arc::new(SiddhiQueryContext::new(
        make_app_ctx(),
        name.to_string(),
        None,
    ))
}

fn empty_ctx(query: &str) -> ExpressionParserContext {
    ExpressionParserContext {
        siddhi_app_context: make_app_ctx(),
        siddhi_query_context: make_query_ctx(query),
        stream_meta_map: HashMap::new(),
        table_meta_map: HashMap::new(),
        window_meta_map: HashMap::new(),
        aggregation_meta_map: HashMap::new(),
        state_meta_map: HashMap::new(),
        stream_positions: HashMap::new(),
        default_source: "dummy".to_string(),
        query_name: query,
    }
}

#[test]
fn test_cast_function() {
    let ctx = empty_ctx("cast");
    let expr = Expression::function_no_ns(
        "cast".to_string(),
        vec![
            Expression::value_int(5),
            Expression::value_string("string".to_string()),
        ],
    );
    let exec = parse_expression(&expr, &ctx).expect("parse failed");
    assert_eq!(exec.get_return_type(), AttrType::STRING);
    let result = exec.execute(None);
    assert_eq!(result, Some(AttributeValue::String("5".to_string())));
}

#[test]
fn test_concat_and_length_functions() {
    let ctx = empty_ctx("concat_len");
    let concat_expr = Expression::function_no_ns(
        "concat".to_string(),
        vec![
            Expression::value_string("ab".to_string()),
            Expression::value_string("cd".to_string()),
        ],
    );
    let concat_exec = parse_expression(&concat_expr, &ctx).unwrap();
    let concat_res = concat_exec.execute(None);
    assert_eq!(concat_res, Some(AttributeValue::String("abcd".to_string())));

    let len_expr = Expression::function_no_ns(
        "length".to_string(),
        vec![Expression::value_string("hello".to_string())],
    );
    let len_exec = parse_expression(&len_expr, &ctx).unwrap();
    let len_res = len_exec.execute(None);
    assert_eq!(len_res, Some(AttributeValue::Int(5)));
}

#[test]
fn test_current_timestamp_function() {
    let ctx = empty_ctx("current_ts");
    let expr = Expression::function_no_ns("currentTimestamp".to_string(), vec![]);
    let exec = parse_expression(&expr, &ctx).unwrap();
    assert_eq!(exec.get_return_type(), AttrType::LONG);
    let val = exec.execute(None);
    match val {
        Some(AttributeValue::Long(_)) => {}
        _ => panic!("expected long"),
    }
}

#[test]
fn test_format_date_function() {
    let ctx = empty_ctx("format_date");
    let expr = Expression::function_no_ns(
        "formatDate".to_string(),
        vec![
            Expression::value_long(0),
            Expression::value_string("%Y".to_string()),
        ],
    );
    let exec = parse_expression(&expr, &ctx).unwrap();
    let result = exec.execute(None);
    assert_eq!(result, Some(AttributeValue::String("1970".to_string())));
}

#[test]
fn test_round_and_sqrt_functions() {
    let ctx = empty_ctx("math");
    let round_expr =
        Expression::function_no_ns("round".to_string(), vec![Expression::value_double(3.7)]);
    let round_exec = parse_expression(&round_expr, &ctx).unwrap();
    assert_eq!(round_exec.execute(None), Some(AttributeValue::Double(4.0)));

    let sqrt_expr = Expression::function_no_ns("sqrt".to_string(), vec![Expression::value_int(16)]);
    let sqrt_exec = parse_expression(&sqrt_expr, &ctx).unwrap();
    assert_eq!(sqrt_exec.execute(None), Some(AttributeValue::Double(4.0)));
}
