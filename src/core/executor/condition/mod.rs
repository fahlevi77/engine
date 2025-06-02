// siddhi_rust/src/core/executor/condition/mod.rs

pub mod and_expression_executor;
pub mod or_expression_executor;
pub mod not_expression_executor;
pub mod compare_expression_executor;
pub mod is_null_expression_executor;
pub mod in_expression_executor;
pub mod bool_expression_executor; // Added

pub use self::and_expression_executor::AndExpressionExecutor;
pub use self::or_expression_executor::OrExpressionExecutor;
pub use self::not_expression_executor::NotExpressionExecutor;
pub use self::compare_expression_executor::CompareExpressionExecutor;
// ConditionCompareOperator is re-exported from query_api::expression::condition::CompareOperator
// No need to re-export it here unless it's a new local definition.
pub use self::is_null_expression_executor::IsNullExpressionExecutor;
pub use self::in_expression_executor::InExpressionExecutor;
pub use self::bool_expression_executor::BoolExpressionExecutor; // Added

// ConditionExpressionExecutor.java is an abstract class that these extend.
// In Rust, they all implement the ExpressionExecutor trait.
// No direct equivalent of ConditionExpressionExecutor itself is needed as a struct/trait here,
// unless it had specific methods beyond ExpressionExecutor that all condition executors shared.
// The main commonality is that they all return Attribute::Type::BOOL.
