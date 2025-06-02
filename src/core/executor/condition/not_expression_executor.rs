// siddhi_rust/src/core/executor/condition/not_expression_executor.rs
// Corresponds to io.siddhi.core.executor.condition.NotConditionExpressionExecutor
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::query_api::definition::Attribute;

#[derive(Debug)]
pub struct NotExpressionExecutor {
    executor: Box<dyn ExpressionExecutor>
}

impl NotExpressionExecutor {
    pub fn new(executor: Box<dyn ExpressionExecutor>) -> Result<Self, String> {
        if executor.get_return_type() != Attribute::Type::BOOL {
            return Err(format!(
                "Operand for NOT executor returns {:?} instead of BOOL",
                executor.get_return_type()
            ));
        }
        Ok(Self { executor })
    }
}

impl ExpressionExecutor for NotExpressionExecutor {
   fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
       match self.executor.execute(event) {
           Some(AttributeValue::Bool(b)) => Some(AttributeValue::Bool(!b)),
           Some(AttributeValue::Null) => Some(AttributeValue::Bool(true)), // NOT NULL is true in some SQL contexts,
                                                                      // but Siddhi might treat NOT NULL as NULL or error.
                                                                      // Java: Boolean result = (Boolean) conditionExecutor.execute(event);
                                                                      // if (result == Boolean.TRUE) return Boolean.FALSE; else return Boolean.TRUE;
                                                                      // This implies nulls are treated as false by the inner condition.
                                                                      // So, if inner is null (false), NOT makes it true.
           None => None, // Propagate error/no-value
           _ => None, // Type error
       }
   }
   fn get_return_type(&self) -> Attribute::Type {
       Attribute::Type::BOOL
   }
   // fn clone_executor(&self) -> Box<dyn ExpressionExecutor> {
   //     Box::new(NotExpressionExecutor {
   //         executor: self.executor.clone_executor(),
   //     })
   // }
}
