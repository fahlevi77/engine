// siddhi_rust/src/core/executor/condition/in_expression_executor.rs
// Corresponds to io.siddhi.core.executor.condition.InConditionExpressionExecutor
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::query_api::definition::attribute::Type as ApiAttributeType; // Import Type enum
use std::sync::Arc; // For SiddhiAppContext in clone_executor
use crate::core::config::siddhi_app_context::SiddhiAppContext; // For clone_executor
use crate::core::table::Table;

#[derive(Debug)]
pub struct InExpressionExecutor {
    // Java fields:
    // private final int streamEventSize;
    // private final boolean isMatchingEventAStateEvent;
    // private final int matchingStreamIndex;
    // private final CompiledCondition compiledCondition;
    // private Table table;

    // Simplified implementation fields
    value_executor: Box<dyn ExpressionExecutor>, // Executes the expression whose value is checked for "IN"
    table_id: String,
    siddhi_app_context: Arc<SiddhiAppContext>,
}

impl InExpressionExecutor {
    pub fn new(
        value_executor: Box<dyn ExpressionExecutor>,
        table_id: String,
        siddhi_app_context: Arc<SiddhiAppContext>,
    ) -> Self {
        Self {
            value_executor,
            table_id,
            siddhi_app_context,
        }
    }
}

impl ExpressionExecutor for InExpressionExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let value_to_check = self.value_executor.execute(event);
        let value = match value_to_check {
            Some(v) => {
                if matches!(v, AttributeValue::Null) {
                    return Some(AttributeValue::Bool(false));
                }
                v
            }
            None => return Some(AttributeValue::Bool(false)),
        };

        let table_opt = self
            .siddhi_app_context
            .get_siddhi_context()
            .get_table(&self.table_id);

        if let Some(table) = table_opt {
            let key = vec![value.clone()];
            let contains = table.contains(&key);
            Some(AttributeValue::Bool(contains))
        } else {
            // If the table is not found, treat as false
            Some(AttributeValue::Bool(false))
        }
    }

    fn get_return_type(&self) -> ApiAttributeType {
        ApiAttributeType::BOOL
    }

    fn clone_executor(&self, siddhi_app_context: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
        Box::new(InExpressionExecutor::new(
            self.value_executor.clone_executor(siddhi_app_context),
            self.table_id.clone(),
            Arc::clone(siddhi_app_context),
        ))
    }
}
