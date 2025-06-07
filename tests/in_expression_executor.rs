use siddhi_rust::core::executor::condition::InExpressionExecutor;
use siddhi_rust::core::executor::constant_expression_executor::ConstantExpressionExecutor;
use siddhi_rust::core::executor::expression_executor::ExpressionExecutor;
use siddhi_rust::core::table::{InMemoryTable, Table};
use siddhi_rust::core::config::siddhi_app_context::SiddhiAppContext;
use siddhi_rust::core::config::siddhi_context::SiddhiContext;
use siddhi_rust::query_api::siddhi_app::SiddhiApp;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::query_api::definition::attribute::Type as ApiAttributeType;
use std::sync::Arc;

fn make_context_with_table() -> Arc<SiddhiAppContext> {
    let ctx = Arc::new(SiddhiAppContext::new(
        Arc::new(SiddhiContext::default()),
        "test_app".to_string(),
        Arc::new(SiddhiApp::new("test".to_string())),
        String::new(),
    ));
    let table: Arc<dyn Table> = Arc::new(InMemoryTable::new());
    table.insert(&[AttributeValue::Int(1)]);
    ctx.get_siddhi_context().add_table("MyTable".to_string(), table);
    ctx
}

#[test]
fn test_in_true() {
    let app_ctx = make_context_with_table();
    let const_exec = Box::new(ConstantExpressionExecutor::new(
        AttributeValue::Int(1),
        ApiAttributeType::INT,
    ));
    let in_exec = InExpressionExecutor::new(const_exec, "MyTable".to_string(), Arc::clone(&app_ctx));
    let result = in_exec.execute(None);
    assert_eq!(result, Some(AttributeValue::Bool(true)));
}

#[test]
fn test_in_false() {
    let app_ctx = make_context_with_table();
    let const_exec = Box::new(ConstantExpressionExecutor::new(
        AttributeValue::Int(5),
        ApiAttributeType::INT,
    ));
    let in_exec = InExpressionExecutor::new(const_exec, "MyTable".to_string(), Arc::clone(&app_ctx));
    let result = in_exec.execute(None);
    assert_eq!(result, Some(AttributeValue::Bool(false)));
}

#[test]
fn test_in_clone() {
    let app_ctx = make_context_with_table();
    let const_exec = Box::new(ConstantExpressionExecutor::new(
        AttributeValue::Int(1),
        ApiAttributeType::INT,
    ));
    let in_exec = InExpressionExecutor::new(const_exec, "MyTable".to_string(), Arc::clone(&app_ctx));
    let cloned = in_exec.clone_executor(&app_ctx);
    let result = cloned.execute(None);
    assert_eq!(result, Some(AttributeValue::Bool(true)));
}
