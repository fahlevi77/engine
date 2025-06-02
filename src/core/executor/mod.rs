pub mod expression_executor;
pub mod constant_expression_executor;
pub mod variable_expression_executor;
pub mod condition;
pub mod math;
pub mod function;  // Added for function executors
// pub mod incremental; // For incremental aggregation executors

pub use self::expression_executor::ExpressionExecutor;
pub use self::constant_expression_executor::ConstantExpressionExecutor;
pub use self::variable_expression_executor::{VariableExpressionExecutor, VariablePosition, EventDataArrayType};
pub use self::condition::*;
pub use self::math::*;
pub use self::function::*; // Re-export function executors
// AttributeDynamicResolveType was in the prompt for VariableExpressionExecutor, but not used in my simplified impl yet.
// If it were, it would be exported:
// pub use self::variable_expression_executor::AttributeDynamicResolveType;
