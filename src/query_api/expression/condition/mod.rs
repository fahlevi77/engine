// Corresponds to the package io.siddhi.query.api.expression.condition

pub mod and;
pub mod compare;
pub mod in_op; // Renamed from in to in_op
pub mod is_null;
pub mod not;
pub mod or;

pub use self::and::And;
pub use self::compare::{Compare, Operator as CompareOperator}; // Re-export Operator enum too
pub use self::in_op::InOp;
pub use self::is_null::IsNull;
pub use self::not::Not;
pub use self::or::Or;

// Comments from before, updated for SiddhiElement composition:
// Each condition struct (And, Or, etc.) will contain:
// - siddhi_element: SiddhiElement
// - Other specific fields, often Box<Expression> for operands.
//
// The Expression enum (in expression/expression.rs) will have variants
// like Expression::And(Box<And>), etc.
//
// IsNull struct has Option fields to handle its different construction paths.
