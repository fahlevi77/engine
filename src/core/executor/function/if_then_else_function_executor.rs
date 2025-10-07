// eventflux_rust/src/core/executor/function/if_then_else_function_executor.rs
// Corresponds to io.eventflux.core.executor.function.IfThenElseFunctionExecutor
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::query_api::definition::attribute::Type as ApiAttributeType; // Import Type enum

#[derive(Debug)]
pub struct IfThenElseFunctionExecutor {
    condition_executor: Box<dyn ExpressionExecutor>,
    then_expression_executor: Box<dyn ExpressionExecutor>,
    else_expression_executor: Box<dyn ExpressionExecutor>,
    return_type: ApiAttributeType, // Determined by compatible type of then/else
}

impl IfThenElseFunctionExecutor {
    pub fn new(
        cond_exec: Box<dyn ExpressionExecutor>,
        then_exec: Box<dyn ExpressionExecutor>,
        else_exec: Box<dyn ExpressionExecutor>,
    ) -> Result<Self, String> {
        if cond_exec.get_return_type() != ApiAttributeType::BOOL {
            return Err(format!(
                "Condition for IfThenElse must return BOOL, found {:?}",
                cond_exec.get_return_type()
            ));
        }

        let then_type = then_exec.get_return_type();
        let else_type = else_exec.get_return_type();

        // Java's IfThenElseFunctionExecutor checks:
        // if (!attributeExpressionExecutors[1].getReturnType().equals(attributeExpressionExecutors[2].getReturnType()))
        // This means then_type and else_type must be strictly equal.
        // The prompt's example allowed one to be OBJECT, which is more flexible for coercion.
        // Sticking to stricter Java logic for now.
        if then_type != else_type {
            return Err(format!(
                "Then/Else branches in IfThenElseFunctionExecutor must have the same type, found {then_type:?} and {else_type:?}"
            ));
        }
        let return_type = then_type; // Since they must be equal

        Ok(Self {
            condition_executor: cond_exec,
            then_expression_executor: then_exec,
            else_expression_executor: else_exec,
            return_type,
        })
    }
}

impl ExpressionExecutor for IfThenElseFunctionExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        // Java's IfThenElseFunctionExecutor overrides the main execute(ComplexEvent event) method
        // and then calls its own execute(Object[] data) method.
        // The logic is: execute condition, if true, execute then_branch, else execute else_branch.

        let condition_result = self.condition_executor.execute(event);

        match condition_result {
            Some(AttributeValue::Bool(true)) => self.then_expression_executor.execute(event),
            Some(AttributeValue::Bool(false)) => self.else_expression_executor.execute(event),
            Some(AttributeValue::Null) => self.else_expression_executor.execute(event), // Condition is null, Java treats as false
            None => None, // Error or no value from condition executor
            _ => {
                // Condition did not evaluate to a Bool or Null (type error)
                // This should be caught by constructor validation.
                // log_error!("IfThenElse condition did not return a boolean.");
                None
            }
        }
    }

    fn get_return_type(&self) -> ApiAttributeType {
        self.return_type
    }

    fn clone_executor(
        &self,
        eventflux_app_context: &std::sync::Arc<
            crate::core::config::eventflux_app_context::EventFluxAppContext,
        >,
    ) -> Box<dyn ExpressionExecutor> {
        Box::new(
            IfThenElseFunctionExecutor::new(
                self.condition_executor
                    .clone_executor(eventflux_app_context),
                self.then_expression_executor
                    .clone_executor(eventflux_app_context),
                self.else_expression_executor
                    .clone_executor(eventflux_app_context),
            )
            .expect("Cloning IfThenElseFunctionExecutor failed"),
        ) // unwrap() is risky if new can fail for other reasons
    }
}
