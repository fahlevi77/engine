use crate::core::config::eventflux_app_context::EventFluxAppContext;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::query_api::definition::attribute::Type as ApiAttributeType;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct EventTimestampFunctionExecutor {
    event_executor: Option<Box<dyn ExpressionExecutor>>,
}

impl EventTimestampFunctionExecutor {
    pub fn new(event_executor: Option<Box<dyn ExpressionExecutor>>) -> Self {
        Self { event_executor }
    }
}

impl ExpressionExecutor for EventTimestampFunctionExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        if self.event_executor.is_some() {
            // simplified: not supporting explicit event argument yet
            None
        } else {
            Some(AttributeValue::Long(event?.get_timestamp()))
        }
    }

    fn get_return_type(&self) -> ApiAttributeType {
        ApiAttributeType::LONG
    }

    fn clone_executor(&self, ctx: &Arc<EventFluxAppContext>) -> Box<dyn ExpressionExecutor> {
        Box::new(EventTimestampFunctionExecutor {
            event_executor: self.event_executor.as_ref().map(|e| e.clone_executor(ctx)),
        })
    }
}
