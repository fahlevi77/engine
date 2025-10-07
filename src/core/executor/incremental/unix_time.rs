// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::core::config::eventflux_app_context::EventFluxAppContext;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::query_api::definition::attribute::Type as ApiAttributeType;
use chrono::{DateTime, NaiveDateTime};
use std::sync::Arc;

#[derive(Debug)]
pub struct IncrementalUnixTimeFunctionExecutor {
    arg_executor: Box<dyn ExpressionExecutor>,
}

impl IncrementalUnixTimeFunctionExecutor {
    pub fn new(arg: Box<dyn ExpressionExecutor>) -> Result<Self, String> {
        if arg.get_return_type() != ApiAttributeType::STRING {
            return Err("timestampInMilliseconds() expects a STRING argument".into());
        }
        Ok(Self { arg_executor: arg })
    }

    pub fn parse_timestamp(s: &str) -> Option<i64> {
        let s = s.trim();
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
            return Some(dt.and_utc().timestamp_millis());
        }
        if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S %z") {
            return Some(dt.timestamp_millis());
        }
        None
    }
}

impl ExpressionExecutor for IncrementalUnixTimeFunctionExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let val = self.arg_executor.execute(event)?;
        let ts_str = match val {
            AttributeValue::String(s) => s,
            _ => return None,
        };
        let ts = Self::parse_timestamp(&ts_str)?;
        Some(AttributeValue::Long(ts))
    }

    fn get_return_type(&self) -> ApiAttributeType {
        ApiAttributeType::LONG
    }

    fn clone_executor(&self, ctx: &Arc<EventFluxAppContext>) -> Box<dyn ExpressionExecutor> {
        Box::new(Self {
            arg_executor: self.arg_executor.clone_executor(ctx),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::value::AttributeValue;
    use crate::core::executor::constant_expression_executor::ConstantExpressionExecutor;
    use std::sync::Arc;

    #[test]
    fn test_unix_time_exec() {
        let arg = Box::new(ConstantExpressionExecutor::new(
            AttributeValue::String("2017-06-01 04:05:50".to_string()),
            ApiAttributeType::STRING,
        ));
        let exec = IncrementalUnixTimeFunctionExecutor::new(arg).unwrap();
        let res = exec.execute(None);
        let dt = chrono::NaiveDateTime::parse_from_str("2017-06-01 04:05:50", "%Y-%m-%d %H:%M:%S")
            .unwrap();
        let expected = dt.and_utc().timestamp_millis();
        assert_eq!(res, Some(AttributeValue::Long(expected)));
    }

    #[test]
    fn test_clone() {
        let arg = Box::new(ConstantExpressionExecutor::new(
            AttributeValue::String("2017-06-01 04:05:50".to_string()),
            ApiAttributeType::STRING,
        ));
        let exec = IncrementalUnixTimeFunctionExecutor::new(arg).unwrap();
        let ctx = Arc::new(EventFluxAppContext::default_for_testing());
        let cloned = exec.clone_executor(&ctx);
        let res = cloned.execute(None);
        let dt = chrono::NaiveDateTime::parse_from_str("2017-06-01 04:05:50", "%Y-%m-%d %H:%M:%S")
            .unwrap();
        let expected = dt.and_utc().timestamp_millis();
        assert_eq!(res, Some(AttributeValue::Long(expected)));
    }
}
