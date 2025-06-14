// siddhi_rust/src/core/executor/function/string_functions.rs
use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::query_api::definition::attribute::Type as ApiAttributeType;
use std::sync::Arc;

#[derive(Debug)]
pub struct LengthFunctionExecutor {
    expr: Box<dyn ExpressionExecutor>,
}

impl LengthFunctionExecutor {
    pub fn new(expr: Box<dyn ExpressionExecutor>) -> Result<Self, String> {
        if expr.get_return_type() != ApiAttributeType::STRING {
            return Err("length() requires a STRING argument".to_string());
        }
        Ok(Self { expr })
    }
}

impl ExpressionExecutor for LengthFunctionExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        match self.expr.execute(event)? {
            AttributeValue::String(s) => Some(AttributeValue::Int(s.len() as i32)),
            AttributeValue::Null => Some(AttributeValue::Null),
            _ => None,
        }
    }

    fn get_return_type(&self) -> ApiAttributeType {
        ApiAttributeType::INT
    }

    fn clone_executor(&self, ctx: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
        Box::new(LengthFunctionExecutor {
            expr: self.expr.clone_executor(ctx),
        })
    }
}

#[derive(Debug)]
pub struct ConcatFunctionExecutor {
    executors: Vec<Box<dyn ExpressionExecutor>>,
}

impl ConcatFunctionExecutor {
    pub fn new(executors: Vec<Box<dyn ExpressionExecutor>>) -> Result<Self, String> {
        if executors.is_empty() {
            return Err("concat() requires at least one argument".to_string());
        }
        for e in &executors {
            if e.get_return_type() != ApiAttributeType::STRING {
                return Err("concat() arguments must be STRING".to_string());
            }
        }
        Ok(Self { executors })
    }
}

impl ExpressionExecutor for ConcatFunctionExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let mut result = String::new();
        for e in &self.executors {
            match e.execute(event)? {
                AttributeValue::String(s) => result.push_str(&s),
                AttributeValue::Null => return Some(AttributeValue::Null),
                _ => return None,
            }
        }
        Some(AttributeValue::String(result))
    }

    fn get_return_type(&self) -> ApiAttributeType {
        ApiAttributeType::STRING
    }

    fn clone_executor(&self, ctx: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
        Box::new(ConcatFunctionExecutor {
            executors: self
                .executors
                .iter()
                .map(|e| e.clone_executor(ctx))
                .collect(),
        })
    }
}
