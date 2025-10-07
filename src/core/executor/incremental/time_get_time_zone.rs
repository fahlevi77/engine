// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::core::config::eventflux_app_context::EventFluxAppContext;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::query_api::definition::attribute::Type as ApiAttributeType;
use chrono::{DateTime, Local};
use std::sync::Arc;

#[derive(Debug)]
pub struct IncrementalTimeGetTimeZone {
    arg_executor: Option<Box<dyn ExpressionExecutor>>,
}

impl IncrementalTimeGetTimeZone {
    pub fn new(arg: Option<Box<dyn ExpressionExecutor>>) -> Result<Self, String> {
        if let Some(ref ex) = arg {
            if ex.get_return_type() != ApiAttributeType::STRING {
                return Err("getTimeZone() argument must be STRING".into());
            }
        }
        Ok(Self { arg_executor: arg })
    }

    fn extract_tz(s: &str) -> Option<String> {
        let s = s.trim();
        if chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").is_ok() {
            return Some("+00:00".to_string());
        }
        if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S %z") {
            return Some(dt.offset().to_string());
        }
        None
    }

    fn system_offset() -> String {
        Local::now().offset().to_string()
    }
}

impl ExpressionExecutor for IncrementalTimeGetTimeZone {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        if let Some(ref ex) = self.arg_executor {
            let val = ex.execute(event)?;
            if let AttributeValue::String(s) = val {
                return Self::extract_tz(&s).map(AttributeValue::String);
            } else {
                return None;
            }
        }
        Some(AttributeValue::String(Self::system_offset()))
    }

    fn get_return_type(&self) -> ApiAttributeType {
        ApiAttributeType::STRING
    }

    fn clone_executor(&self, ctx: &Arc<EventFluxAppContext>) -> Box<dyn ExpressionExecutor> {
        Box::new(Self {
            arg_executor: self.arg_executor.as_ref().map(|e| e.clone_executor(ctx)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executor::constant_expression_executor::ConstantExpressionExecutor;

    #[test]
    fn test_get_time_zone_from_string() {
        let arg = Box::new(ConstantExpressionExecutor::new(
            AttributeValue::String("2017-06-01 04:05:50 +05:00".to_string()),
            ApiAttributeType::STRING,
        ));
        let exec = IncrementalTimeGetTimeZone::new(Some(arg)).unwrap();
        let res = exec.execute(None);
        assert_eq!(res, Some(AttributeValue::String("+05:00".to_string())));
    }
}
