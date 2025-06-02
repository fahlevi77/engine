// siddhi_rust/src/core/executor/constant_expression_executor.rs
// Corresponds to io.siddhi.core.executor.ConstantExpressionExecutor
use super::expression_executor::ExpressionExecutor;
use crate::core::event::complex_event::ComplexEvent; // Trait
use crate::core::event::value::AttributeValue; // Enum for actual data
use crate::query_api::definition::Attribute; // For Attribute::Type enum

#[derive(Debug, Clone)] // Constant values are cloneable
pub struct ConstantExpressionExecutor {
    value: AttributeValue, // Stores the constant value directly using our AttributeValue enum
    return_type: Attribute::Type, // Stores the type of the constant
}

impl ConstantExpressionExecutor {
    pub fn new(value: AttributeValue, return_type: Attribute::Type) -> Self {
        // In a more robust implementation, we might validate that the AttributeValue variant
        // matches the provided Attribute::Type.
        // e.g., if return_type is Attribute::Type::INT, value should be AttributeValue::Int.
        // For now, assuming the caller provides consistent types, as in ExpressionParser.
        Self { value, return_type }
    }

    // Getter for the value if needed (not in Java ExpressionExecutor interface)
    pub fn get_value(&self) -> &AttributeValue {
        &self.value
    }
}

impl ExpressionExecutor for ConstantExpressionExecutor {
    fn execute(&self, _event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        // Constants don't depend on the event, so _event is ignored.
        // Return a clone of the stored constant value.
        Some(self.value.clone())
    }

    fn get_return_type(&self) -> Attribute::Type {
        self.return_type
    }

    // fn clone_executor(&self) -> Box<dyn ExpressionExecutor> {
    //     Box::new(self.clone())
    // }
}
