// siddhi_rust/src/core/executor/function/instance_of_checkers.rs
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::query_api::definition::Attribute; // For Attribute::Type enum

// Macro to define InstanceOf*FunctionExecutor structs and their impls
macro_rules! define_instance_of_executor {
    ($struct_name:ident, $attribute_variant:pat) => {
        #[derive(Debug)] // Clone not straightforward due to Box<dyn ExpressionExecutor>
        pub struct $struct_name {
            executor: Box<dyn ExpressionExecutor>
        }

        impl $struct_name {
            // Java constructor checks attributeExpressionExecutors.length != 1
            // For simplicity, new takes one executor directly.
            pub fn new(executor: Box<dyn ExpressionExecutor>) -> Result<Self, String> {
                // The input executor can return any type.
                Ok(Self { executor })
            }
        }

        impl ExpressionExecutor for $struct_name {
            fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
                // Java execute(Object data, S state) -> data instanceof <Type>
                // Here, `data` is the result of self.executor.execute(event)
                match self.executor.execute(event) {
                    Some($attribute_variant(_)) => Some(AttributeValue::Bool(true)),
                    Some(AttributeValue::Null) => Some(AttributeValue::Bool(false)), // null is not an instance of any specific type
                    Some(_) => Some(AttributeValue::Bool(false)), // Other type
                    None => Some(AttributeValue::Bool(false)), // No value from executor, not an instance
                }
            }

            fn get_return_type(&self) -> Attribute::Type {
                Attribute::Type::BOOL
            }

            // fn clone_executor(&self) -> Box<dyn ExpressionExecutor> {
            //     Box::new(Self { // Using Self works within macro
            //         executor: self.executor.clone_executor(),
            //     })
            // }
        }
    };
}

define_instance_of_executor!(InstanceOfBooleanExpressionExecutor, AttributeValue::Bool);
define_instance_of_executor!(InstanceOfStringExpressionExecutor, AttributeValue::String);
define_instance_of_executor!(InstanceOfIntegerExpressionExecutor, AttributeValue::Int); // Java: Integer
define_instance_of_executor!(InstanceOfLongExpressionExecutor, AttributeValue::Long);
define_instance_of_executor!(InstanceOfFloatExpressionExecutor, AttributeValue::Float);
define_instance_of_executor!(InstanceOfDoubleExpressionExecutor, AttributeValue::Double);

// InstanceOfObjectFunctionExecutor in Java would check if `data != null`.
// This is slightly different as our AttributeValue::Object can hold None.
// For `instanceOfObject(arg)`, it should return true if `arg` is not siddhi `null` AND its type is OBJECT.
// However, the Java `InstanceOf*` functions check against specific Java types.
// `instanceOfObject` is not a standard one in the provided list, usually it's specific types.
// If it means "is this an AttributeValue::Object variant (regardless of inner Option)":
// define_instance_of_executor!(InstanceOfObjectExpressionExecutor, AttributeValue::Object);
// If it means "is this not AttributeValue::Null":
// (This would be a different function like `isNotNull`)
