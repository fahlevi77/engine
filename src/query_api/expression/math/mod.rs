// SPDX-License-Identifier: MIT OR Apache-2.0

// Corresponds to the package io.eventflux.query.api.expression.math

pub mod add;
pub mod divide;
pub mod mod_op;
pub mod multiply;
pub mod subtract; // Renamed from mod to mod_op to avoid keyword clash

pub use self::add::Add;
pub use self::divide::Divide;
pub use self::mod_op::ModOp;
pub use self::multiply::Multiply;
pub use self::subtract::Subtract;

// Comments from before are still relevant:
// Each math operation struct (Add, Subtract, etc.) will contain:
// - eventflux_element: EventFluxElement (updated from direct fields)
// - left_value: Box<Expression>
// - right_value: Box<Expression>
//
// The Expression enum will be defined in the parent module (expression/expression.rs)
// and will have variants like Expression::Add(Box<Add>), etc.
// Box<T> is used because these are recursive types via the Expression enum.
