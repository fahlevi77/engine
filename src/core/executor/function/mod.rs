// siddhi_rust/src/core/executor/function/mod.rs

pub mod coalesce_function_executor;
pub mod if_then_else_function_executor;
pub mod instance_of_checkers;
pub mod uuid_function_executor;
// pub mod function_executor_base; // If a base struct for stateful functions is created later

pub use self::coalesce_function_executor::CoalesceFunctionExecutor;
pub use self::if_then_else_function_executor::IfThenElseFunctionExecutor;
pub use self::instance_of_checkers::*; // Export all instance_of structs (e.g., InstanceOfBooleanExpressionExecutor)
pub use self::uuid_function_executor::UuidFunctionExecutor;

// Other function executors to be added here:
// pub mod cast_function_executor;
// pub mod convert_function_executor;
// pub mod script_function_executor;
// ... etc.
