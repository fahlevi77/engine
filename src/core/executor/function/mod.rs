// siddhi_rust/src/core/executor/function/mod.rs

pub mod coalesce_function_executor;
pub mod if_then_else_function_executor;
pub mod instance_of_checkers;
pub mod uuid_function_executor;
pub mod scalar_function_executor; // Added
pub mod cast_function_executor;
pub mod convert_function_executor;
pub mod event_timestamp_function_executor;
pub mod builtin_wrapper;
pub mod string_functions;
pub mod date_functions;
pub mod math_functions;

// pub mod function_executor_base; // If a base struct for stateful functions is created later

pub use self::coalesce_function_executor::CoalesceFunctionExecutor;
pub use self::if_then_else_function_executor::IfThenElseFunctionExecutor;
pub use self::instance_of_checkers::*;
pub use self::uuid_function_executor::UuidFunctionExecutor;
pub use self::scalar_function_executor::ScalarFunctionExecutor; // Added
pub use self::cast_function_executor::CastFunctionExecutor;
pub use self::convert_function_executor::ConvertFunctionExecutor;
pub use self::event_timestamp_function_executor::EventTimestampFunctionExecutor;
pub use self::builtin_wrapper::{BuiltinScalarFunction, BuiltinBuilder};
pub use self::string_functions::{ConcatFunctionExecutor, LengthFunctionExecutor};
pub use self::date_functions::{CurrentTimestampFunctionExecutor, FormatDateFunctionExecutor};
pub use self::math_functions::{RoundFunctionExecutor, SqrtFunctionExecutor};

// Other function executors to be added here:
// pub mod cast_function_executor;
// pub mod convert_function_executor;
// pub mod script_function_executor;
// ... etc.
