use eventflux_rust::core::event::value::AttributeValue;
use eventflux_rust::core::executor::constant_expression_executor::ConstantExpressionExecutor;
use eventflux_rust::core::executor::expression_executor::ExpressionExecutor;
use eventflux_rust::core::executor::function::{
    CoalesceFunctionExecutor, IfThenElseFunctionExecutor, InstanceOfStringExpressionExecutor,
    UuidFunctionExecutor,
};
use eventflux_rust::query_api::definition::attribute::Type as AttrType;

#[test]
fn test_coalesce_function() {
    let exec = CoalesceFunctionExecutor::new(vec![
        Box::new(ConstantExpressionExecutor::new(
            AttributeValue::Null,
            AttrType::STRING,
        )),
        Box::new(ConstantExpressionExecutor::new(
            AttributeValue::String("x".into()),
            AttrType::STRING,
        )),
    ])
    .unwrap();
    assert_eq!(exec.execute(None), Some(AttributeValue::String("x".into())));
}

#[test]
fn test_if_then_else_function() {
    let cond = Box::new(ConstantExpressionExecutor::new(
        AttributeValue::Bool(true),
        AttrType::BOOL,
    ));
    let then_exec = Box::new(ConstantExpressionExecutor::new(
        AttributeValue::Int(1),
        AttrType::INT,
    ));
    let else_exec = Box::new(ConstantExpressionExecutor::new(
        AttributeValue::Int(2),
        AttrType::INT,
    ));
    let exec = IfThenElseFunctionExecutor::new(cond, then_exec, else_exec).unwrap();
    assert_eq!(exec.execute(None), Some(AttributeValue::Int(1)));
}

#[test]
fn test_uuid_function() {
    let exec = UuidFunctionExecutor::new();
    match exec.execute(None) {
        Some(AttributeValue::String(s)) => assert_eq!(s.len(), 36),
        _ => panic!("expected uuid string"),
    }
}

#[test]
fn test_instance_of_string() {
    let inner = Box::new(ConstantExpressionExecutor::new(
        AttributeValue::String("hi".into()),
        AttrType::STRING,
    ));
    let exec = InstanceOfStringExpressionExecutor::new(inner).unwrap();
    assert_eq!(exec.execute(None), Some(AttributeValue::Bool(true)));
}
