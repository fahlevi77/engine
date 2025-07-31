use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::query_api::definition::attribute::Type as ApiAttributeType;
use super::unix_time::IncrementalUnixTimeFunctionExecutor;
use std::sync::Arc;

#[derive(Debug)]
pub struct IncrementalStartTimeEndTimeFunctionExecutor {
    start_executor: Box<dyn ExpressionExecutor>,
    end_executor: Box<dyn ExpressionExecutor>,
}

impl IncrementalStartTimeEndTimeFunctionExecutor {
    pub fn new(
        start_exec: Box<dyn ExpressionExecutor>,
        end_exec: Box<dyn ExpressionExecutor>,
    ) -> Result<Self, String> {
        Ok(Self {
            start_executor: start_exec,
            end_executor: end_exec,
        })
    }
}

impl ExpressionExecutor for IncrementalStartTimeEndTimeFunctionExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let s_val = self.start_executor.execute(event)?;
        let start = match s_val {
            AttributeValue::Long(v) => v,
            AttributeValue::String(ref st) => IncrementalUnixTimeFunctionExecutor::parse_timestamp(st)?,
            _ => return None,
        };
        let e_val = self.end_executor.execute(event)?;
        let end = match e_val {
            AttributeValue::Long(v) => v,
            AttributeValue::String(ref st) => IncrementalUnixTimeFunctionExecutor::parse_timestamp(st)?,
            _ => return None,
        };
        if start >= end {
            return None;
        }
        Some(AttributeValue::Object(Some(Box::new(vec![
            AttributeValue::Long(start),
            AttributeValue::Long(end),
        ]))))
    }

    fn get_return_type(&self) -> ApiAttributeType {
        ApiAttributeType::OBJECT
    }

    fn clone_executor(&self, ctx: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
        Box::new(Self {
            start_executor: self.start_executor.clone_executor(ctx),
            end_executor: self.end_executor.clone_executor(ctx),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executor::constant_expression_executor::ConstantExpressionExecutor;

    #[test]
    fn test_start_end() {
        let s = Box::new(ConstantExpressionExecutor::new(
            AttributeValue::Long(1000),
            ApiAttributeType::LONG,
        ));
        let e = Box::new(ConstantExpressionExecutor::new(
            AttributeValue::Long(2000),
            ApiAttributeType::LONG,
        ));
        let exec = IncrementalStartTimeEndTimeFunctionExecutor::new(s, e).unwrap();
        let res = exec.execute(None).unwrap();
        if let AttributeValue::Object(Some(b)) = res {
            let v = b.downcast_ref::<Vec<AttributeValue>>().unwrap();
            assert_eq!(v[0], AttributeValue::Long(1000));
            assert_eq!(v[1], AttributeValue::Long(2000));
        } else {
            panic!("unexpected result");
        }
    }
}
