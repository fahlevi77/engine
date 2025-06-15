use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::query_api::aggregation::time_period::Duration;
use crate::query_api::definition::attribute::Type as ApiAttributeType;
use chrono::{Datelike, Timelike, Utc, TimeZone};
use std::sync::Arc;

#[derive(Debug)]
pub struct IncrementalAggregateBaseTimeFunctionExecutor {
    time_executor: Box<dyn ExpressionExecutor>,
    duration_executor: Box<dyn ExpressionExecutor>,
}

impl IncrementalAggregateBaseTimeFunctionExecutor {
    pub fn new(
        time_exec: Box<dyn ExpressionExecutor>,
        dur_exec: Box<dyn ExpressionExecutor>,
    ) -> Result<Self, String> {
        if time_exec.get_return_type() != ApiAttributeType::LONG {
            return Err("aggregateBaseTime() first argument must be LONG".into());
        }
        if dur_exec.get_return_type() != ApiAttributeType::STRING {
            return Err("aggregateBaseTime() second argument must be STRING".into());
        }
        Ok(Self {
            time_executor: time_exec,
            duration_executor: dur_exec,
        })
    }

    fn start_time(ts: i64, dur: Duration) -> i64 {
        match dur {
            Duration::Seconds => ts - ts % dur.to_millis() as i64,
            Duration::Minutes => ts - ts % dur.to_millis() as i64,
            Duration::Hours => {
                let dt = chrono::DateTime::<Utc>::from_timestamp_millis(ts).unwrap();
                Utc.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), dt.hour(), 0, 0)
                    .unwrap()
                    .timestamp_millis()
            }
            Duration::Days => {
                let dt = chrono::DateTime::<Utc>::from_timestamp_millis(ts).unwrap();
                Utc.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0)
                    .unwrap()
                    .timestamp_millis()
            }
            Duration::Months => {
                let dt = chrono::DateTime::<Utc>::from_timestamp_millis(ts).unwrap();
                Utc.with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0)
                    .unwrap()
                    .timestamp_millis()
            }
            Duration::Years => {
                let dt = chrono::DateTime::<Utc>::from_timestamp_millis(ts).unwrap();
                Utc.with_ymd_and_hms(dt.year(), 1, 1, 0, 0, 0)
                    .unwrap()
                    .timestamp_millis()
            }
        }
    }
}

impl ExpressionExecutor for IncrementalAggregateBaseTimeFunctionExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let time_val = self.time_executor.execute(event)?;
        let ts = match time_val {
            AttributeValue::Long(v) => v,
            AttributeValue::Int(v) => v as i64,
            _ => return None,
        };
        let dur_val = self.duration_executor.execute(event)?;
        let dur_str = match dur_val {
            AttributeValue::String(s) => s.to_uppercase(),
            _ => return None,
        };
        let dur = match dur_str.as_str() {
            "SECONDS" => Duration::Seconds,
            "MINUTES" => Duration::Minutes,
            "HOURS" => Duration::Hours,
            "DAYS" => Duration::Days,
            "MONTHS" => Duration::Months,
            "YEARS" => Duration::Years,
            _ => return None,
        };
        let start = Self::start_time(ts, dur);
        Some(AttributeValue::Long(start))
    }

    fn get_return_type(&self) -> ApiAttributeType {
        ApiAttributeType::LONG
    }

    fn clone_executor(&self, ctx: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
        Box::new(Self {
            time_executor: self.time_executor.clone_executor(ctx),
            duration_executor: self.duration_executor.clone_executor(ctx),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executor::constant_expression_executor::ConstantExpressionExecutor;

    #[test]
    fn test_basic_hour() {
        let time_exec = Box::new(ConstantExpressionExecutor::new(
            AttributeValue::Long(1_672_531_234_000),
            ApiAttributeType::LONG,
        ));
        let dur_exec = Box::new(ConstantExpressionExecutor::new(
            AttributeValue::String("HOURS".to_string()),
            ApiAttributeType::STRING,
        ));
        let exec = IncrementalAggregateBaseTimeFunctionExecutor::new(time_exec, dur_exec).unwrap();
        let res = exec.execute(None);
        let expected = 1_672_531_200_000i64; // start of hour
        assert_eq!(res, Some(AttributeValue::Long(expected)));
    }
}
