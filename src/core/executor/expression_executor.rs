// siddhi_rust/src/core/executor/expression_executor.rs
// Corresponds to io.siddhi.core.executor.ExpressionExecutor (interface)

use crate::core::event::complex_event::ComplexEvent; // The ComplexEvent trait
use crate::core::event::value::AttributeValue; // The enum for actual data values
use crate::query_api::definition::Attribute; // For Attribute::Type enum
use std::fmt::Debug;
// use std::sync::Arc; // Not needed for trait definition itself

// ExpressionExecutor is an interface in Java. In Rust, it's a trait.
// It's responsible for executing a parsed Siddhi expression and returning a value.
pub trait ExpressionExecutor: Debug + Send + Sync + 'static {
    // `event` is Option<&dyn ComplexEvent> because some expressions (like constants)
    // don't need an event to be evaluated. Others (like variables) do.
    // The Option<&dyn ComplexEvent> allows passing None for constant expressions.
    // The return type is Option<AttributeValue> because an expression might evaluate to null,
    // or an error could occur during execution (though errors might be better handled by Result).
    // Java returns Object, which can be null.
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue>;

    // Returns the data type of the value that this executor will return.
    fn get_return_type(&self) -> Attribute::Type;

    // Optional: A method to clone the executor.
    // This is useful if the execution plan needs to be cloned, for example, for partitioning.
    // Each concrete executor would implement this to clone itself.
    // fn clone_executor(&self) -> Box<dyn ExpressionExecutor>;
}

// If ExpressionExecutor instances need to be cloned as trait objects:
// impl Clone for Box<dyn ExpressionExecutor> {
//     fn clone(&self) -> Self {
//         self.clone_executor()
//     }
// }
