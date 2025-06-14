// siddhi_rust/src/core/executor/function/date_functions.rs
use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::query_api::definition::attribute::Type as ApiAttributeType;
use chrono::{NaiveDateTime, Utc};
use std::sync::Arc;

#[derive(Debug, Default, Clone)]
pub struct CurrentTimestampFunctionExecutor;

impl ExpressionExecutor for CurrentTimestampFunctionExecutor {
    fn execute(&self, _event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let millis = Utc::now().timestamp_millis();
        Some(AttributeValue::Long(millis))
    }

    fn get_return_type(&self) -> ApiAttributeType {
        ApiAttributeType::LONG
    }

    fn clone_executor(&self, _ctx: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
        Box::new(self.clone())
    }
}

#[derive(Debug)]
pub struct FormatDateFunctionExecutor {
    timestamp_executor: Box<dyn ExpressionExecutor>,
    pattern: String,
}

impl FormatDateFunctionExecutor {
    pub fn new(
        timestamp_executor: Box<dyn ExpressionExecutor>,
        pattern_executor: Box<dyn ExpressionExecutor>,
    ) -> Result<Self, String> {
        if pattern_executor.get_return_type() != ApiAttributeType::STRING {
            return Err("formatDate pattern must be STRING".to_string());
        }
        let pattern = match pattern_executor.execute(None) {
            Some(AttributeValue::String(s)) => s,
            _ => return Err("formatDate pattern must be constant string".to_string()),
        };
        Ok(Self {
            timestamp_executor,
            pattern,
        })
    }
}

impl ExpressionExecutor for FormatDateFunctionExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let ts_val = self.timestamp_executor.execute(event)?;
        let ts = match ts_val {
            AttributeValue::Long(v) => v,
            AttributeValue::Int(v) => v as i64,
            AttributeValue::Null => return Some(AttributeValue::Null),
            _ => return None,
        };
        let ndt = chrono::DateTime::<Utc>::from_timestamp_millis(ts)?.naive_utc();
        let formatted = ndt.format(&self.pattern).to_string();
        Some(AttributeValue::String(formatted))
    }

    fn get_return_type(&self) -> ApiAttributeType {
        ApiAttributeType::STRING
    }

    fn clone_executor(&self, ctx: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
        Box::new(FormatDateFunctionExecutor {
            timestamp_executor: self.timestamp_executor.clone_executor(ctx),
            pattern: self.pattern.clone(),
        })
    }
}
