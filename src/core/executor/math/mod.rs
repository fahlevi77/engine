// siddhi_rust/src/core/executor/math/mod.rs

pub mod common; // Added common module for CoerceNumeric
pub mod add;
pub mod subtract;
pub mod multiply;
pub mod divide;
pub mod mod_expression_executor; // Changed from mod_op to match filename

pub use self::add::AddExpressionExecutor;
pub use self::subtract::SubtractExpressionExecutor;
pub use self::multiply::MultiplyExpressionExecutor;
pub use self::divide::DivideExpressionExecutor;
pub use self::mod_expression_executor::ModExpressionExecutor;
// CoerceNumeric is pub(super) in common.rs, so not re-exported here.
