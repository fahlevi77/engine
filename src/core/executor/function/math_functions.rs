// siddhi_rust/src/core/executor/function/math_functions.rs
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::query_api::definition::attribute::Type as ApiAttributeType;
use crate::core::config::siddhi_app_context::SiddhiAppContext;
use std::sync::Arc;

fn to_f64(val: &AttributeValue) -> Option<f64> {
    match val {
        AttributeValue::Int(v) => Some(*v as f64),
        AttributeValue::Long(v) => Some(*v as f64),
        AttributeValue::Float(v) => Some(*v as f64),
        AttributeValue::Double(v) => Some(*v),
        _ => None,
    }
}

#[derive(Debug)]
pub struct SqrtFunctionExecutor {
    value_executor: Box<dyn ExpressionExecutor>,
}

impl SqrtFunctionExecutor {
    pub fn new(value_executor: Box<dyn ExpressionExecutor>) -> Result<Self, String> {
        Ok(Self { value_executor })
    }
}

impl ExpressionExecutor for SqrtFunctionExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let val = self.value_executor.execute(event)?;
        match val {
            AttributeValue::Null => Some(AttributeValue::Null),
            _ => {
                let num = to_f64(&val)?;
                Some(AttributeValue::Double(num.sqrt()))
            }
        }
    }

    fn get_return_type(&self) -> ApiAttributeType {
        ApiAttributeType::DOUBLE
    }

    fn clone_executor(&self, ctx: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
        Box::new(SqrtFunctionExecutor {
            value_executor: self.value_executor.clone_executor(ctx),
        })
    }
}

#[derive(Debug)]
pub struct RoundFunctionExecutor {
    value_executor: Box<dyn ExpressionExecutor>,
}

impl RoundFunctionExecutor {
    pub fn new(value_executor: Box<dyn ExpressionExecutor>) -> Result<Self, String> {
        Ok(Self { value_executor })
    }
}

impl ExpressionExecutor for RoundFunctionExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let val = self.value_executor.execute(event)?;
        match val {
            AttributeValue::Null => Some(AttributeValue::Null),
            _ => {
                let num = to_f64(&val)?;
                Some(AttributeValue::Double(num.round()))
            }
        }
    }

    fn get_return_type(&self) -> ApiAttributeType {
        ApiAttributeType::DOUBLE
    }

    fn clone_executor(&self, ctx: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
        Box::new(RoundFunctionExecutor {
            value_executor: self.value_executor.clone_executor(ctx),
        })
    }
}
