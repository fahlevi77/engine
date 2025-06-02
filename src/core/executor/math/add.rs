// siddhi_rust/src/core/executor/math/add.rs
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::query_api::definition::Attribute; // For Attribute::Type enum
use super::common::CoerceNumeric; // Use CoerceNumeric from common.rs

#[derive(Debug)] // Clone not straightforward due to Box<dyn ExpressionExecutor>
pub struct AddExpressionExecutor {
    left_executor: Box<dyn ExpressionExecutor>,
    right_executor: Box<dyn ExpressionExecutor>,
    return_type: Attribute::Type, // Determined at construction based on operand types
}

impl AddExpressionExecutor {
    pub fn new(left: Box<dyn ExpressionExecutor>, right: Box<dyn ExpressionExecutor>) -> Result<Self, String> {
        let left_type = left.get_return_type();
        let right_type = right.get_return_type();

        // Simplified type promotion logic (matches common SQL-like behavior)
        // TODO: Align strictly with Siddhi's specific type promotion rules.
        let return_type = match (left_type, right_type) {
            (Attribute::Type::STRING, _) | (_, Attribute::Type::STRING) => {
                // String concatenation if enabled, or error. Siddhi does not support + for strings.
                return Err(format!("String concatenation with '+' is not supported. Use str:concat(). Found input types {:?} and {:?}.", left_type, right_type));
            }
            (Attribute::Type::BOOL, _) | (_, Attribute::Type::BOOL) => {
                return Err(format!("Arithmetic addition not supported for BOOL types. Found input types {:?} and {:?}.", left_type, right_type));
            }
            (Attribute::Type::OBJECT, _) | (_, Attribute::Type::OBJECT) => {
                 return Err(format!("Arithmetic addition not supported for OBJECT types. Found input types {:?} and {:?}.", left_type, right_type));
            }
            (Attribute::Type::DOUBLE, _) | (_, Attribute::Type::DOUBLE) => Attribute::Type::DOUBLE,
            (Attribute::Type::FLOAT, _) | (_, Attribute::Type::FLOAT) => Attribute::Type::FLOAT,
            (Attribute::Type::LONG, _) | (_, Attribute::Type::LONG) => Attribute::Type::LONG,
            (Attribute::Type::INT, _) | (_, Attribute::Type::INT) => Attribute::Type::INT,
            // If one is INT and other is not yet covered (e.g. custom numeric type if any)
            // This default case should ideally not be reached if all numeric types are handled above.
            _ => return Err(format!("Addition not supported for incompatible types: {:?} and {:?}", left_type, right_type)),

        };
        Ok(Self { left_executor: left, right_executor: right, return_type })
    }
}

impl ExpressionExecutor for AddExpressionExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let left_val_opt = self.left_executor.execute(event);
        let right_val_opt = self.right_executor.execute(event);

        match (left_val_opt, right_val_opt) {
            (Some(left_val), Some(right_val)) => {
                // Handle NULL propagation: if either operand is Null, result is Null.
                if matches!(left_val, AttributeValue::Null) || matches!(right_val, AttributeValue::Null) {
                    return Some(AttributeValue::Null);
                }

                // Coerce operands to the target return_type and perform addition
                match self.return_type {
                    Attribute::Type::INT => {
                        let l = left_val.to_i32_or_err_str("Add")?;
                        let r = right_val.to_i32_or_err_str("Add")?;
                        Some(AttributeValue::Int(l.wrapping_add(r))) // Use wrapping_add for overflow resilience like Java
                    }
                    Attribute::Type::LONG => {
                        let l = left_val.to_i64_or_err_str("Add")?;
                        let r = right_val.to_i64_or_err_str("Add")?;
                        Some(AttributeValue::Long(l.wrapping_add(r)))
                    }
                    Attribute::Type::FLOAT => {
                        let l = left_val.to_f32_or_err_str("Add")?;
                        let r = right_val.to_f32_or_err_str("Add")?;
                        Some(AttributeValue::Float(l + r))
                    }
                    Attribute::Type::DOUBLE => {
                        let l = left_val.to_f64_or_err_str("Add")?;
                        let r = right_val.to_f64_or_err_str("Add")?;
                        Some(AttributeValue::Double(l + r))
                    }
                    _ => {
                        // This should not be reached if constructor validation is correct.
                        // log_error!("Addition resulted in unexpected return type: {:?}", self.return_type);
                        None
                    }
                }
            }
            _ => None, // If any operand execution fails (returns None)
        }
    }
    fn get_return_type(&self) -> Attribute::Type { self.return_type }

    // fn clone_executor(&self) -> Box<dyn ExpressionExecutor> {
    //     Box::new(AddExpressionExecutor {
    //         left_executor: self.left_executor.clone_executor(),
    //         right_executor: self.right_executor.clone_executor(),
    //         return_type: self.return_type,
    //     })
    // }
}

// CoerceNumeric trait and impl removed from here, now in common.rs
