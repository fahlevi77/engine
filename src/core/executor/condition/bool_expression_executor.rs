// siddhi_rust/src/core/executor/condition/bool_expression_executor.rs
// Corresponds to io.siddhi.core.executor.condition.BoolConditionExpressionExecutor
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::query_api::definition::Attribute;

#[derive(Debug)]
pub struct BoolExpressionExecutor {
    condition_executor: Box<dyn ExpressionExecutor>
}

impl BoolExpressionExecutor {
    pub fn new(condition_executor: Box<dyn ExpressionExecutor>) -> Result<Self, String> {
        if condition_executor.get_return_type() != Attribute::Type::BOOL {
            return Err(format!(
                "BoolConditionExpressionExecutor expects an inner executor returning BOOL, but got {:?}",
                condition_executor.get_return_type()
            ));
        }
        Ok(Self { condition_executor })
    }
}

impl ExpressionExecutor for BoolExpressionExecutor {
   fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
       match self.condition_executor.execute(event) {
           Some(AttributeValue::Bool(b)) => Some(AttributeValue::Bool(b)),
           Some(AttributeValue::Null) => Some(AttributeValue::Bool(false)), // null is false in boolean context
           None => Some(AttributeValue::Bool(false)), // Error/no-value treated as false
           _ => {
               // This case should ideally be prevented by the constructor check,
               // but as a fallback, non-bool result is treated as false.
               Some(AttributeValue::Bool(false))
           }
       }
   }

   fn get_return_type(&self) -> Attribute::Type {
       Attribute::Type::BOOL
   }

   // fn clone_executor(&self) -> Box<dyn ExpressionExecutor> {
   //     Box::new(BoolExpressionExecutor {
   //         condition_executor: self.condition_executor.clone_executor(),
   //     })
   // }
}
