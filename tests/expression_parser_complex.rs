use siddhi_rust::core::util::parser::{parse_expression, ExpressionParserContext};
use siddhi_rust::query_api::expression::{Expression, Variable};
use siddhi_rust::query_api::expression::condition::compare::Operator as CompareOp;
use siddhi_rust::query_api::definition::{StreamDefinition, attribute::Type as AttrType};
use siddhi_rust::core::event::stream::meta_stream_event::MetaStreamEvent;
use siddhi_rust::core::config::siddhi_app_context::SiddhiAppContext;
use siddhi_rust::core::config::siddhi_context::SiddhiContext;
use siddhi_rust::core::config::siddhi_query_context::SiddhiQueryContext;
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

fn make_query_ctx(name: &str) -> Arc<SiddhiQueryContext> {
    Arc::new(SiddhiQueryContext::new(make_app_ctx(), name.to_string(), None))
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
        siddhi_query_context: make_query_ctx("Q1"),
        stream_meta_map: map,
        table_meta_map: HashMap::new(),
        window_meta_map: HashMap::new(),
        state_meta_map: HashMap::new(),
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
        siddhi_query_context: make_query_ctx("Q2"),
        stream_meta_map: HashMap::new(),
        table_meta_map: HashMap::new(),
        window_meta_map: HashMap::new(),
        state_meta_map: HashMap::new(),
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
        siddhi_query_context: make_query_ctx("Q3"),
        stream_meta_map: map,
        table_meta_map: HashMap::new(),
        window_meta_map: HashMap::new(),
        state_meta_map: HashMap::new(),
        default_source: "A".to_string(),
        query_name: "Q3",
    };

    let var_b = Variable::new("missing".to_string()).of_stream("A".to_string());
    let expr = Expression::Variable(var_b);
    let err = parse_expression(&expr, &ctx).unwrap_err();
    assert!(err.contains("Q3"));
}

#[test]
fn test_table_variable_resolution() {
    use siddhi_rust::query_api::definition::TableDefinition;
    let table = TableDefinition::new("T".to_string())
        .attribute("val".to_string(), AttrType::INT);
    let stream_equiv = Arc::new(StreamDefinition::new("T".to_string()).attribute("val".to_string(), AttrType::INT));
    let meta = Arc::new(MetaStreamEvent::new_for_single_input(stream_equiv));
    let mut table_map = HashMap::new();
    table_map.insert("T".to_string(), Arc::clone(&meta));

    let ctx = ExpressionParserContext {
        siddhi_app_context: make_app_ctx(),
        siddhi_query_context: make_query_ctx("Q4"),
        stream_meta_map: HashMap::new(),
        table_meta_map: table_map,
        window_meta_map: HashMap::new(),
        state_meta_map: HashMap::new(),
        default_source: "T".to_string(),
        query_name: "Q4",
    };

    let var = Variable::new("val".to_string()).of_stream("T".to_string());
    let expr = Expression::Variable(var);
    let exec = parse_expression(&expr, &ctx).unwrap();
    assert_eq!(exec.get_return_type(), AttrType::INT);
}

#[test]
fn test_custom_udf_plus_one() {
    use siddhi_rust::core::siddhi_manager::SiddhiManager;
    use siddhi_rust::core::executor::expression_executor::ExpressionExecutor;
    use siddhi_rust::core::executor::function::scalar_function_executor::ScalarFunctionExecutor;
    use siddhi_rust::core::event::value::AttributeValue;
    #[derive(Debug, Default)]
    struct PlusOneFn { arg: Option<Box<dyn ExpressionExecutor>> }

    impl Clone for PlusOneFn {
        fn clone(&self) -> Self { Self { arg: None } }
    }

    impl ExpressionExecutor for PlusOneFn {
        fn execute(&self, event: Option<&dyn siddhi_rust::core::event::complex_event::ComplexEvent>) -> Option<AttributeValue> {
            let v = self.arg.as_ref()?.execute(event)?;
            match v { AttributeValue::Int(i) => Some(AttributeValue::Int(i+1)), _ => None }
        }
        fn get_return_type(&self) -> AttrType { AttrType::INT }
        fn clone_executor(&self, _ctx:&Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> { Box::new(self.clone()) }
    }

    impl ScalarFunctionExecutor for PlusOneFn {
        fn init(&mut self, args:&Vec<Box<dyn ExpressionExecutor>>, _ctx:&Arc<SiddhiAppContext>) -> Result<(), String> {
            if args.len()!=1 { return Err("plusOne expects one argument".to_string()); }
            self.arg = Some(args[0].clone_executor(_ctx));
            Ok(())
        }
        fn destroy(&mut self) {}
        fn get_name(&self) -> String { "plusOne".to_string() }
        fn clone_scalar_function(&self) -> Box<dyn ScalarFunctionExecutor> { Box::new(self.clone()) }
    }

    let manager = SiddhiManager::new();
    manager.add_scalar_function_factory("plusOne".to_string(), Box::new(PlusOneFn::default()));

    let app_ctx = Arc::new(SiddhiAppContext::new(
        manager.siddhi_context(),
        "app".to_string(),
        Arc::new(SiddhiApp::new("app".to_string())),
        String::new(),
    ));
    let q_ctx = Arc::new(SiddhiQueryContext::new(Arc::clone(&app_ctx), "Q5".to_string(), None));

    let ctx = ExpressionParserContext {
        siddhi_app_context: app_ctx,
        siddhi_query_context: q_ctx,
        stream_meta_map: HashMap::new(),
        table_meta_map: HashMap::new(),
        window_meta_map: HashMap::new(),
        state_meta_map: HashMap::new(),
        default_source: "dummy".to_string(),
        query_name: "Q5",
    };

    let expr = Expression::function_no_ns("plusOne".to_string(), vec![Expression::value_int(4)]);
    let exec = parse_expression(&expr, &ctx).unwrap();
    let res = exec.execute(None);
    assert_eq!(res, Some(AttributeValue::Int(5)));
}
